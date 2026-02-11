# Audit Report: Hostname Lookup Mechanism

## Scope
-   `src/net.rs`: `resolve_hostname` function.
-   `Cargo.toml`: `dns-lookup` dependency.

## Findings

### Resolution Mechanism
RageScanner uses the standard system resolver via the `dns-lookup` Rust crate. Specifically, it calls `dns_lookup::lookup_addr`, which performs a reverse DNS lookup.

### Data Sources
Since the lookup is delegated to the Windows operating system (using `getnameinfo`), it follows the standard Windows name resolution order:
1.  **Local Hosts File**: Checks `C:\Windows\System32\drivers\etc\hosts` first. If an IP is mapped there, that name is returned.
2.  **DNS Cache / DNS Server**: Queries configured DNS servers.
3.  **Local Name Services**: If DNS fails, it may fallback to **LLMNR** (Link-Local Multicast Name Resolution) or **NetBIOS** (if enabled) to find the computer name of the target IP on the local network.

### Result Filtering
The implementation includes a check (line 117 in `src/net.rs`) that returns `None` if the "hostname" returned is identical to the IP string. This prevents the UI from showing redundant IP addresses in the Hostname column when no name is found.

## Risk Level
**Low**. The current approach is standard and leverages the OS's native capabilities, ensuring consistency with how other Windows tools (like `ping -a`) behave.

## Proposed Strategy
No changes are required as the current implementation correctly addresses both "computer name" and "hosts file" resolution by leveraging the system resolver.

---
🛑 **Audit Complete.** Please review the findings above. Reply with **"Proceed"** to implement any specific changes or if you have further questions.
