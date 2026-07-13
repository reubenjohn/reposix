#!/usr/bin/env python3
"""TokenWorld Confluence inspect / list / reparent / delete helper.

Sanctioned mutation target ONLY: the TokenWorld space (key REPOSIX, id 360450),
owned by the project owner (docs/reference/testing-targets.md). This tool exists
because the durable-fixture parent/child invariant
(contract.rs::contract_confluence_live_hierarchy) can be broken by heavy session
churn and turn CI red; it gives the next agent one named command instead of
opaque curl pipelines.

Protected durable-fixture ids that this tool REFUSES to delete: 7766017, 7798785.

Credentials from env (load via `set -a; source .env; set +a`):
  ATLASSIAN_EMAIL, ATLASSIAN_API_KEY, REPOSIX_CONFLUENCE_TENANT

Usage:
  confluence_tokenworld.py inspect <page_id>
  confluence_tokenworld.py list [--space-id 360450]
  confluence_tokenworld.py reparent <child_id> <parent_id>
  confluence_tokenworld.py delete <page_id>        # refuses protected ids
"""
from __future__ import annotations

import json
import os
import sys
import urllib.request
import urllib.error
from base64 import b64encode

PROTECTED = {"7766017", "7798785"}
TOKENWORLD_SPACE_ID = "360450"


def _base() -> str:
    tenant = os.environ["REPOSIX_CONFLUENCE_TENANT"]
    return f"https://{tenant}.atlassian.net"


def _auth_header() -> str:
    email = os.environ["ATLASSIAN_EMAIL"]
    token = os.environ["ATLASSIAN_API_KEY"]
    raw = f"{email}:{token}".encode()
    return "Basic " + b64encode(raw).decode()


def _req(method: str, path: str, body: dict | None = None) -> tuple[int, dict | str]:
    url = _base() + path
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", _auth_header())
    req.add_header("Accept", "application/json")
    if data is not None:
        req.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(req) as resp:
            txt = resp.read().decode()
            return resp.status, (json.loads(txt) if txt else "")
    except urllib.error.HTTPError as e:
        txt = e.read().decode()
        try:
            return e.code, json.loads(txt)
        except Exception:
            return e.code, txt


def cmd_inspect(page_id: str) -> int:
    status, d = _req("GET", f"/wiki/api/v2/pages/{page_id}")
    if status != 200:
        print(f"HTTP {status}: {d}")
        return 1
    print(json.dumps({
        "id": d.get("id"),
        "title": d.get("title"),
        "status": d.get("status"),
        "parentId": d.get("parentId"),
        "parentType": d.get("parentType"),
        "spaceId": d.get("spaceId"),
        "version": (d.get("version") or {}).get("number"),
    }, indent=2))
    return 0


def cmd_list(space_id: str) -> int:
    path = f"/wiki/api/v2/spaces/{space_id}/pages?limit=250&status=current"
    rows = []
    while path:
        status, d = _req("GET", path)
        if status != 200:
            print(f"HTTP {status}: {d}")
            return 1
        for p in d.get("results", []):
            rows.append(p)
        nxt = (d.get("_links") or {}).get("next")
        path = nxt if nxt else None
    rows.sort(key=lambda p: int(p["id"]))
    for p in rows:
        prot = " [PROTECTED]" if str(p["id"]) in PROTECTED else ""
        print(f'{p["id"]:>10}  parent={str(p.get("parentId")):>10}  {p.get("title")}{prot}')
    print(f"--- {len(rows)} current pages in space {space_id} ---")
    return 0


def cmd_reparent(child_id: str, parent_id: str) -> int:
    status, d = _req("GET", f"/wiki/api/v2/pages/{child_id}?body-format=storage")
    if status != 200:
        print(f"GET child HTTP {status}: {d}")
        return 1
    ver = (d.get("version") or {}).get("number", 0)
    body = {
        "id": child_id,
        "status": "current",
        "title": d.get("title"),
        "spaceId": d.get("spaceId"),
        "parentId": parent_id,
        "body": {
            "representation": "storage",
            "value": ((d.get("body") or {}).get("storage") or {}).get("value", ""),
        },
        "version": {"number": ver + 1, "message": "restore durable-fixture parent link"},
    }
    status, d = _req("PUT", f"/wiki/api/v2/pages/{child_id}", body)
    if status != 200:
        print(f"PUT HTTP {status}: {d}")
        return 1
    print(f"reparented {child_id} -> parent {d.get('parentId')} (version {(d.get('version') or {}).get('number')})")
    return 0


def cmd_restore(page_id: str) -> int:
    """Un-trash a page. v2 DELETE trashes; restore uses the v1 status flip."""
    status, d = _req("GET", f"/wiki/api/v2/pages/{page_id}")
    if status != 200:
        print(f"GET HTTP {status}: {d}")
        return 1
    cur_status = d.get("status")
    if cur_status == "current":
        print(f"{page_id} already current (no-op)")
        return 0
    ver = (d.get("version") or {}).get("number", 1)
    body = {"id": str(page_id), "status": "current", "version": {"number": ver + 1}}
    status, d = _req("PUT", f"/wiki/rest/api/content/{page_id}?status=trashed", body)
    if status != 200:
        print(f"restore PUT HTTP {status}: {d}")
        return 1
    print(f"restored {page_id} -> status current (version {(d.get('version') or {}).get('number')})")
    return 0


def cmd_delete(page_id: str) -> int:
    if str(page_id) in PROTECTED:
        print(f"REFUSING to delete protected durable-fixture id {page_id}")
        return 2
    status, d = _req("DELETE", f"/wiki/api/v2/pages/{page_id}")
    if status not in (200, 204):
        print(f"DELETE HTTP {status}: {d}")
        return 1
    print(f"deleted {page_id} (HTTP {status})")
    return 0


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 2
    cmd = argv[1]
    if cmd == "inspect" and len(argv) == 3:
        return cmd_inspect(argv[2])
    if cmd == "list":
        space = argv[3] if len(argv) == 4 and argv[2] == "--space-id" else TOKENWORLD_SPACE_ID
        return cmd_list(space)
    if cmd == "reparent" and len(argv) == 4:
        return cmd_reparent(argv[2], argv[3])
    if cmd == "restore" and len(argv) == 3:
        return cmd_restore(argv[2])
    if cmd == "delete" and len(argv) == 3:
        return cmd_delete(argv[2])
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
