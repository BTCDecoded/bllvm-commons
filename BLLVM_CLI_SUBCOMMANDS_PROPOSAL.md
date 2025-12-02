# BLLVM Binary Subcommands - Complete Proposal

## Current State
- `bllvm` (default) - Starts the node
- Uses `clap` for CLI parsing
- Has RPC methods: `getblockchaininfo`, `getnetworkinfo`, `getpeerinfo`

## Proposed Subcommand Structure

### Core Commands

#### `bllvm status`
**Purpose:** Comprehensive node status  
**Output:** Block height, sync status, peer count, uptime, network  
**RPC:** `getblockchaininfo`, `getnetworkinfo`  
**Naming:** ✅ Standard, clear

#### `bllvm health`
**Purpose:** Quick health check (for monitoring/scripts)  
**Output:** Exit code 0 if healthy, 1 if unhealthy  
**RPC:** `getblockchaininfo` (check if responding)  
**Naming:** ✅ Standard, clear

#### `bllvm version`
**Purpose:** Version and build information  
**Output:** Version, git commit, build date, enabled features  
**Naming:** ✅ Better than "info" - more specific

### Node Information Commands

#### `bllvm chain`
**Purpose:** Blockchain information  
**Output:** Height, tip hash, difficulty, chainwork, verification progress  
**RPC:** `getblockchaininfo`  
**Naming:** ✅ Clear, matches Bitcoin terminology

#### `bllvm peers`
**Purpose:** Connected peer information  
**Output:** Peer list with addresses, versions, latency  
**RPC:** `getpeerinfo`  
**Naming:** ✅ Standard, clear

#### `bllvm network`
**Purpose:** Network connection information  
**Output:** Connection count, network totals, local addresses  
**RPC:** `getnetworkinfo`  
**Naming:** ✅ Clear, matches RPC method

#### `bllvm sync`
**Purpose:** Sync status and progress  
**Output:** Sync status, progress percentage, blocks remaining  
**RPC:** `getblockchaininfo` (verificationprogress)  
**Naming:** ✅ Standard, clear

### Configuration Commands

#### `bllvm config show`
**Purpose:** Display loaded configuration  
**Output:** Merged config (file + env + CLI) in TOML format  
**Naming:** ✅ Good grouping under `config`

#### `bllvm config validate`
**Purpose:** Validate configuration file  
**Output:** Success/error with details  
**Naming:** ✅ Good grouping

#### `bllvm config path`
**Purpose:** Show config file path (if found)  
**Output:** Path to config file or "not found"  
**Naming:** ✅ Good grouping

### Control Commands

#### `bllvm stop`
**Purpose:** Graceful shutdown (if running via IPC/signal)  
**Output:** Confirmation message  
**Implementation:** Send signal to running process or via RPC  
**Naming:** ✅ Standard, clear

### Advanced Commands

#### `bllvm metrics`
**Purpose:** Performance metrics  
**Output:** CPU, memory, network stats, request counts  
**Implementation:** Internal metrics or RPC stats  
**Naming:** ✅ Standard

#### `bllvm rpc <method> [params]`
**Purpose:** Direct RPC call wrapper  
**Output:** JSON-RPC response  
**Example:** `bllvm rpc getblockchaininfo`  
**Naming:** ✅ Direct, useful for debugging

## Naming Review

### ✅ Good Names (Keep)
- `status` - Standard, clear
- `health` - Standard, clear
- `version` - Better than "info", specific
- `chain` - Clear, matches Bitcoin terminology
- `peers` - Standard, clear
- `network` - Clear, matches RPC
- `sync` - Standard, clear
- `config` - Good grouping prefix
- `stop` - Standard, clear
- `metrics` - Standard
- `rpc` - Direct, useful

### ❌ Avoided Names
- `info` - Too generic (replaced with `version`)
- `node` - Redundant (everything is about the node)
- `query` - Too generic
- `get` - Too generic

## Command Structure

```
bllvm [OPTIONS]                    # Start node (default)
bllvm status                       # Node status
bllvm health                       # Health check
bllvm version                      # Version info
bllvm chain                        # Blockchain info
bllvm peers                        # Peer list
bllvm network                      # Network info
bllvm sync                         # Sync status
bllvm config show                  # Show config
bllvm config validate [PATH]      # Validate config
bllvm config path                  # Config file path
bllvm stop                         # Graceful shutdown
bllvm metrics                      # Performance metrics
bllvm rpc <method> [params]        # Direct RPC call
```

## Implementation Priority

### Phase 1 (Essential)
1. `status` - Most useful, combines multiple RPC calls
2. `health` - Critical for monitoring
3. `version` - Simple, useful for debugging
4. `config show` - Already loads config, just print it

### Phase 2 (High Value)
5. `chain` - Blockchain info (uses existing RPC)
6. `peers` - Peer information (uses existing RPC)
7. `network` - Network info (uses existing RPC)
8. `sync` - Sync status (uses existing RPC)

### Phase 3 (Nice to Have)
9. `config validate` - Validate before starting
10. `config path` - Show where config is loaded from
11. `stop` - Graceful shutdown (requires IPC/signal handling)
12. `metrics` - Performance metrics (requires metrics collection)
13. `rpc` - Direct RPC wrapper (useful for debugging)

## Benefits

1. **No shell dependencies** - Pure Rust, works everywhere
2. **Consistent interface** - All node operations in one binary
3. **Better error handling** - Structured errors vs shell scripts
4. **Works without service** - Can query node even if not systemd service
5. **Scriptable** - Easy to parse JSON output
6. **Type-safe** - Rust enforces correct usage

## Comparison to Shell Scripts

| Feature | Binary Subcommand | Shell Script |
|---------|------------------|--------------|
| Status/Health | ✅ `bllvm status` | ✅ `bllvm.sh status` |
| Config | ✅ `bllvm config show` | ✅ `bllvm.sh config` |
| Install/Update | ❌ Needs file system | ✅ `bllvm.sh install` |
| Service Control | ❌ Needs systemd | ✅ `bllvm.sh restart` |
| Logs | ❌ Needs journalctl | ✅ `bllvm.sh logs` |

**Best of both worlds:** Binary for node operations, scripts for system operations.

