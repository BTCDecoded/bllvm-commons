# Bitcoin Commons Deployment

**Unified CLI for Bitcoin Commons Infrastructure**

---

## 🚀 Quick Start

### Single Command Installation

```bash
cd deployment
chmod +x bllvm.sh

# Install BLLVM node
sudo ./bllvm.sh install bllvm --public-ip YOUR_IP

# Check status
./bllvm.sh status
```

---

## 📖 Documentation

**Full Guide:** See [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md) for complete documentation.

**Quick Reference:**

```bash
# Install components
sudo ./bllvm.sh install [bllvm|experimental|commons] [options]

# Management
./bllvm.sh [status|health|info|logs|config|restart] [component]

# Updates
sudo ./bllvm.sh [update|uninstall] [component]
```

---

## 🎯 Components

- **`bllvm`** - Base BLLVM node (production build)
- **`experimental`** - Experimental node (UTXO commitments, custom features)
- **`commons`** - Governance app (bllvm-commons)

---

## 🔧 Features

- ✅ **Unified CLI** - Single `bllvm.sh` entry point
- ✅ **Native Commands** - Uses `bllvm` binary subcommands
- ✅ **Multi-Machine** - Deploy across separate machines
- ✅ **Auto-Configuration** - Automatic setup with sensible defaults
- ✅ **Health Monitoring** - Built-in health checks
- ✅ **Easy Updates** - Simple update/uninstall process

---

## 📋 What Gets Installed

### BLLVM Node
- Binary: `/opt/bllvm/bllvm`
- Config: `/etc/bllvm/bllvm.toml`
- Data: `/var/lib/bllvm`
- Service: `bllvm.service`

### Experimental Node
- Binary: `/opt/bllvm/bllvm-experimental`
- Config: `/etc/bllvm/bllvm.toml`
- Data: `/var/lib/bllvm`
- Service: `bllvm.service` (uses experimental binary)

### Governance App
- Binary: `/opt/bllvm-commons/bllvm-commons`
- Config: `/etc/bllvm-commons/app.toml`
- Data: `/var/lib/bllvm-commons`
- Service: `bllvm-commons.service`

---

## 🐳 Docker Alternative

For Docker-based deployment, see `docker-compose.yml`:

```bash
docker-compose up -d
```

**Note:** Direct installation (this guide) is recommended for production deployments.

---

## 📚 More Information

- **Full Guide:** [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md)
- **Binary Commands:** See `bllvm --help` after installation
- **Configuration:** See component config files in `/etc/bllvm*/`

---

**Status:** Production Ready  
**Last Updated:** 2024

