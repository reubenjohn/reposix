#!/usr/bin/env python3
"""git-remote-poc — minimal remote helper that supports `stateless-connect`.

Spec ref: Documentation/gitremote-helpers.adoc, "stateless-connect" capability.
For partial-clone support we MUST tunnel protocol v2 (filter is v2-only),
which means we must speak `stateless-connect`, not the older `connect`.

Protocol on stdin/stdout — entirely binary on stdin to avoid the
text/binary mixing footgun where TextIOWrapper.readline() over-reads:

    git -> helper:  capabilities\\n
    helper -> git:  stateless-connect\\n\\n
    git -> helper:  stateless-connect git-upload-pack\\n
    helper -> git:  \\n               (single empty line meaning "ready")
    git <-> helper: bidirectional pkt-line stream until both sides see flush

The helper bridges this stream to a local `git upload-pack --stateless-rpc
--advertise-refs` and then `git upload-pack --stateless-rpc` per the smart-
HTTP convention. Each protocol-v2 client request is a self-contained command
terminated by flush; the helper invokes upload-pack once per command.

This is *exactly* what git-remote-http does over HTTP. We do it over
stdio + a local subprocess, which is what a reposix-style backend would do
once it had translated REST data into a local bare git repo.
"""
import os
import subprocess
import sys


STDIN = sys.stdin.buffer
STDOUT = sys.stdout.buffer


_LOG_FILE = os.environ.get("REPOSIX_POC_LOG")  # if set, also log to this file


def log(msg):
    line = f"[helper pid={os.getpid()}] {msg}\n"
    sys.stderr.write(line)
    sys.stderr.flush()
    if _LOG_FILE:
        try:
            with open(_LOG_FILE, "a") as f:
                f.write(line)
        except OSError:
            pass


def read_text_line():
    """Read one line terminated by \\n from binary stdin.
    Returns the line WITHOUT trailing newline, as a str. None on EOF.
    """
    out = bytearray()
    while True:
        c = STDIN.read(1)
        if not c:
            return None
        if c == b"\n":
            return out.decode("utf-8", errors="replace")
        out += c


def read_pkt_line():
    """Read one pkt-line. Returns (kind, payload_bytes)."""
    hdr = STDIN.read(4)
    if len(hdr) < 4:
        return ("eof", b"")
    try:
        n = int(hdr, 16)
    except ValueError:
        log(f"bad pkt header: {hdr!r}")
        return ("eof", b"")
    if n == 0:
        return ("flush", b"")
    if n == 1:
        return ("delim", b"")
    if n == 2:
        return ("response_end", b"")
    payload = STDIN.read(n - 4)
    return ("data", payload)


def send_advertisement(repo_path):
    """Send the protocol-v2 capability advertisement unsolicited.

    Per gitremote-helpers stateless-connect spec: 'After line feed
    terminating the positive (empty) response, the output of the service
    starts.' The SERVER (helper) speaks first. In smart-HTTP this is the
    GET /info/refs response. We get the same bytes by running
    `git upload-pack --advertise-refs --stateless-rpc <repo>`.
    """
    env = os.environ.copy()
    env["GIT_PROTOCOL"] = "version=2"
    proc = subprocess.run(
        ["git", "upload-pack", "--advertise-refs", "--stateless-rpc", repo_path],
        capture_output=True, env=env,
    )
    if proc.returncode != 0:
        log(f"advertise upload-pack exit {proc.returncode}: {proc.stderr.decode(errors='replace')}")
        return False
    log(f"advertisement: {len(proc.stdout)} bytes; head={proc.stdout[:80]!r}")
    # NOTE: Do NOT append response-end (0002) here. The spec says response
    # messages have response-end after flush. The advertisement is the
    # *initial unsolicited stream*, not a response to a request — it is
    # terminated by flush only. (Empirically, git 2.52 errors out with
    # "expected flush after ref listing" if 0002 is appended.)
    STDOUT.write(proc.stdout)
    STDOUT.flush()
    return True


def proxy_v2_request(repo_path):
    """Read one protocol-v2 client request from stdin (terminated by flush),
    run upload-pack --stateless-rpc on the bare repo, write the response
    (terminated by response-end pkt 0002) to stdout.
    """
    request = bytearray()
    while True:
        kind, payload = read_pkt_line()
        if kind == "eof":
            return False
        if kind == "flush":
            request += b"0000"
            break
        if kind == "delim":
            request += b"0001"
            continue
        if kind == "response_end":
            request += b"0002"
            break
        if kind == "data":
            request += f"{len(payload) + 4:04x}".encode() + payload

    log(f"v2 request ({len(request)} bytes); head={bytes(request[:160])!r}")

    env = os.environ.copy()
    env["GIT_PROTOCOL"] = "version=2"
    proc = subprocess.run(
        ["git", "upload-pack", "--stateless-rpc", repo_path],
        input=bytes(request), capture_output=True, env=env,
    )
    if proc.returncode != 0:
        log(f"upload-pack exit {proc.returncode}: {proc.stderr.decode(errors='replace')}")
        return False

    log(f"  response {len(proc.stdout)} bytes; stderr={proc.stderr[:200]!r}; head={proc.stdout[:200]!r}")
    STDOUT.write(proc.stdout)
    STDOUT.write(b"0002")
    STDOUT.flush()
    return True


def list_for_push(repo_path):
    """Handle `list for-push`. Emit ref listing for the bare repo.

    Format: `<oid> <ref>\\n` lines, `@<symref-target> HEAD` for HEAD, then
    a blank line terminator. For an empty repo we emit `? refs/heads/main`
    (unborn sentinel) + symref — matches what a stateless-connect-less
    helper would produce.
    """
    proc = subprocess.run(
        ["git", "--git-dir", repo_path, "show-ref"],
        capture_output=True,
    )
    out = bytearray()
    # Emit all refs.
    if proc.returncode == 0 and proc.stdout.strip():
        for line in proc.stdout.decode().splitlines():
            oid, ref = line.split(" ", 1)
            out += f"{oid} {ref}\n".encode()
    else:
        out += b"? refs/heads/main\n"
    # HEAD symref pointer.
    head_proc = subprocess.run(
        ["git", "--git-dir", repo_path, "symbolic-ref", "HEAD"],
        capture_output=True,
    )
    if head_proc.returncode == 0:
        head_target = head_proc.stdout.decode().strip()
        out += f"@{head_target} HEAD\n".encode()
    out += b"\n"
    log(f"list-for-push response ({len(out)} bytes): {bytes(out)!r}")
    STDOUT.write(bytes(out))
    STDOUT.flush()


def handle_export(repo_path):
    """Handle `export` command: read fast-import stream from stdin, apply
    to the backing bare repo, then emit ok/error per ref.

    Protocol: per gitremote-helpers.adoc, git sends a fast-export stream
    generated with --use-done-feature. The stream ends with a `done`
    command. We pipe the whole stream into `git fast-import --done`,
    then emit status lines.

    Rejection demo: if REPOSIX_POC_REJECT_PUSH is set, drain the stream
    into /dev/null and emit `error <ref> <message>`. This simulates
    backend-conflict detection.
    """
    reject_reason = os.environ.get("REPOSIX_POC_REJECT_PUSH")
    if reject_reason:
        log(f"export: REJECTING per REPOSIX_POC_REJECT_PUSH={reject_reason!r}")
        # We still must consume the stream so git's fast-export subprocess
        # doesn't get EPIPE mid-write. Read until `done\n` line.
        buf = bytearray()
        while True:
            c = STDIN.read(1)
            if not c:
                break
            buf += c
            if buf.endswith(b"\ndone\n") or buf == b"done\n":
                break
        log(f"export: drained {len(buf)} bytes without applying")
        # Figure out which refs fast-export wanted to update so we can
        # reject each one. Parse `commit refs/heads/<x>` lines.
        refs = []
        for line in buf.decode(errors="replace").splitlines():
            # Only match actual `commit refs/...` ref-declaration lines.
            # A naive `startswith("commit ")` also catches the literal
            # "commit 2 from client" commit-message body inside a `data`
            # section, which caused the POC to emit bogus `ok 2 from
            # client` status lines that git displayed as "unexpected
            # status of 2".
            if line.startswith("commit refs/"):
                ref = line.split(" ", 1)[1].strip()
                if ref not in refs:
                    refs.append(ref)
        if not refs:
            refs = ["refs/heads/main"]
        for ref in refs:
            # Per transport-helper.c parse_ref_push_report: free-form
            # messages are displayed verbatim unless they match one of
            # the canned REJECT_* strings. Use free-form so git prints
            # OUR message.
            STDOUT.write(f"error {ref} {reject_reason}\n".encode())
        STDOUT.write(b"\n")
        STDOUT.flush()
        return

    # Accept path: pipe fast-import stream into `git fast-import` on the
    # bare repo. --done makes fast-import honour the `done` terminator.
    log("export: accepting — spawning git fast-import")
    proc = subprocess.Popen(
        ["git", "--git-dir", repo_path, "fast-import", "--quiet", "--done"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    # Copy bytes from our stdin to fast-import's stdin until we see the
    # `done` line. Record the stream for ref extraction.
    #
    # Read blob/data chunks: fast-export emits `data <n>\n<n raw bytes>`.
    # Byte-by-byte is tolerably slow for POC scale; production should
    # parse pkt framing and copy whole chunks.
    buf = bytearray()
    while True:
        c = STDIN.read(1)
        if not c:
            break
        buf += c
        proc.stdin.write(c)
        if buf.endswith(b"\ndone\n") or buf == b"done\n":
            break
    log(f"export: forwarded {len(buf)} bytes to fast-import; tail={bytes(buf[-80:])!r}")
    proc.stdin.close()
    # NOTE: use wait() + explicit read, not communicate(). communicate()
    # unconditionally calls self.stdin.flush() which raises ValueError on
    # a closed pipe — harmless semantically but poisons the status path.
    try:
        proc.wait(timeout=30)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
    out = proc.stdout.read() if proc.stdout else b""
    err = proc.stderr.read() if proc.stderr else b""
    log(f"fast-import rc={proc.returncode}, stdout={out[:200]!r}, stderr={err[:200]!r}")
    # Sanity: dump bare repo refs after fast-import so we can confirm
    # refs/heads/main was actually updated.
    postcheck = subprocess.run(
        ["git", "--git-dir", repo_path, "show-ref"],
        capture_output=True, text=True,
    )
    log(f"post-fast-import show-ref: {postcheck.stdout!r} (rc={postcheck.returncode})")
    # Determine which refs were pushed, by parsing the stream we copied.
    # Only accept actual `commit refs/...` ref-decl lines — a naive
    # `startswith("commit ")` also matches commit-message bodies like
    # "commit 2 from client" inside a `data` section, which poisons the
    # status response with bogus refs ("ok 2 from client" -> git warns
    # "unexpected status of 2").
    refs = []
    for line in buf.decode(errors="replace").splitlines():
        if line.startswith("commit refs/"):
            ref = line.split(" ", 1)[1].strip()
            if ref not in refs:
                refs.append(ref)
    if not refs:
        refs = ["refs/heads/main"]
    if proc.returncode == 0:
        for ref in refs:
            STDOUT.write(f"ok {ref}\n".encode())
    else:
        for ref in refs:
            STDOUT.write(f"error {ref} fast-import failed\n".encode())
    STDOUT.write(b"\n")
    STDOUT.flush()


def main():
    log(f"invoked: {sys.argv}")
    if len(sys.argv) < 3:
        sys.exit("usage: git-remote-poc <alias> <url>")
    # Per gitremote-helpers.adoc, the "<scheme>::" prefix is STRIPPED before
    # the helper sees the URL. argv[2] is just the bare path.
    repo_path = sys.argv[2]
    log(f"repo={repo_path}")

    while True:
        line = read_text_line()
        if line is None:
            return 0
        log(f"<- {line!r}")
        if line == "capabilities":
            # Hybrid advertisement: stateless-connect for FETCH (partial
            # clone + lazy blobs via protocol v2), export for PUSH (fast-
            # export stream we can intercept per commit).
            # Per transport-helper.c::process_connect_service (line 647),
            # `stateless-connect` is ONLY dispatched when the service name
            # is "git-upload-pack" or "git-upload-archive" — so push
            # (which asks for "git-receive-pack") falls through to the
            # export capability even though BOTH are advertised.
            # The refspec capability is mandatory when advertising export.
            STDOUT.write(b"stateless-connect\n")
            STDOUT.write(b"export\n")
            # Private namespace MUST differ from the public ref name.
            # apply_refspecs() maps refs/heads/main -> refs/poc/main; the
            # private ref tracks the last-known-synced OID so fast-export
            # can compute a delta. With refs/heads/*:refs/heads/* (same
            # namespace), repo_get_oid finds the LOCAL ref as the private
            # shadow, the delta is empty, and fast-export emits only a
            # "reset refs/heads/main / from 0000" stanza — effectively
            # deleting the ref. Confirmed empirically in the POC: see
            # push-path-stateless-connect-findings.md Q4 bug trace.
            STDOUT.write(b"refspec refs/heads/*:refs/poc/*\n")
            STDOUT.write(b"\n")
            STDOUT.flush()
        elif line.startswith("stateless-connect "):
            service = line.split(" ", 1)[1]
            log(f"-> '' (ready), tunneling {service} for {repo_path}")
            STDOUT.write(b"\n")
            STDOUT.flush()
            # Spec: "the output of the service starts" right after the
            # ready newline. Send the v2 advertisement unsolicited.
            if not send_advertisement(repo_path):
                return 1
            while True:
                if not proxy_v2_request(repo_path):
                    break
            return 0
        elif line == "list for-push" or line == "list":
            list_for_push(repo_path)
        elif line == "export":
            handle_export(repo_path)
            # After export, spec allows more commands. Loop to read next.
        elif line == "":
            continue
        else:
            log(f"unsupported: {line!r}")
            return 1


if __name__ == "__main__":
    sys.exit(main())
