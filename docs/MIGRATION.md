# Repository Reorganization Migration Guide

**Date**: October 30, 2025  
**Version**: 1.0

---

## Overview

The GerdsenAI Socrates repository has been reorganized to improve maintainability and discoverability. This guide documents all changes to file locations and provides instructions for updating references.

## Summary of Changes

- **Root directory cleaned**: Reduced from 41+ files to approximately 15 files
- **Documentation organized**: All docs moved to `/docs` with logical subdirectories
- **Scripts organized**: All scripts moved to `/scripts` with platform-specific subdirectories
- **Backward compatibility**: Wrapper scripts and symlinks maintain compatibility

---

## Documentation Changes

### Installation Documentation

| Old Location | New Location | Notes |
|-------------|--------------|-------|
| `INSTALL.md` | `docs/installation/installation-guide.md` | Merged with other installation docs |
| `INSTALLATION.md` | `docs/installation/installation-guide.md` | Merged into comprehensive guide |
| `INSTALLATION_README.md` | `docs/installation/installation-guide.md` | Merged into comprehensive guide |

### Setup Documentation

| Old Location | New Location | Notes |
|-------------|--------------|-------|
| `WINDOWS_SETUP.md` | `docs/setup/windows.md` | Platform-specific setup |
| `MACOS_SETUP.md` | `docs/setup/macos.md` | Platform-specific setup |

### Usage Documentation

| Old Location | New Location | Notes |
|-------------|--------------|-------|
| `USAGE.md` | `docs/usage/user-manual.md` | Content merged |
| `USER_GUIDE.md` | `docs/usage/user-manual.md` | Content merged |
| `COMPREHENSIVE_USER_MANUAL.md` | `docs/usage/user-manual.md` | Primary source, enhanced |

### Development Documentation

| Old Location | New Location | Notes |
|-------------|--------------|-------|
| `CONTRIBUTING.md` | `docs/development/contributing.md` | Development guidelines |
| `TYPESCRIPT_UPGRADE_PATTERNS.md` | `docs/development/typescript-patterns.md` | TypeScript best practices |
| `SIDEBAR_INTEGRATION.md` | `docs/development/sidebar-integration.md` | IDE integration guide |

### Guides

| Old Location | New Location | Notes |
|-------------|--------------|-------|
| `QUICK_START.md` | `docs/guides/quick-start.md` | Getting started guide |
| `TROUBLESHOOTING_GUIDE.md` | `docs/guides/troubleshooting.md` | Problem-solving guide |

### Planning Documentation

| Old Location | New Location | Notes |
|-------------|--------------|-------|
| `TODO.md` | `docs/planning/todo.md` | Task list and roadmap |
| `GerdsenAI Socrates Holistic Optimization Plan.md` | `docs/planning/optimization-plan.md` | Strategic planning |

### AI Documentation

| Old Location | New Location | Notes |
|-------------|--------------|-------|
| `CLAUDE.md` | `docs/ai/claude-integration.md` | Claude AI integration guide |

---

## Script Changes

### Windows Scripts

| Old Location | New Location | Backward Compatibility |
|-------------|--------------|------------------------|
| `GENERATE_INSTALLER.BAT` | `scripts/windows/generate-installer.bat` | N/A (internal use) |
| `INSTALL_DEPENDENCIES.BAT` | `scripts/windows/install-dependencies.bat` | N/A (internal use) |
| `START_APPLICATION.BAT` | `scripts/windows/start-application.bat` | `start-wrapper.bat` in root |
| `WINDOWS_SETUP.BAT` | `scripts/windows/windows-setup.bat` | N/A (internal use) |
| `install.bat` | `scripts/windows/install.bat` | `install-wrapper.bat` in root |

### Unix Scripts

| Old Location | New Location | Backward Compatibility |
|-------------|--------------|------------------------|
| `install.sh` | `scripts/install.sh` | `install-symlink.sh` symlink in root |
| `setup-fix.sh` | `scripts/setup-fix.sh` | `setup-fix-symlink.sh` symlink in root |

---

## New Structure

### Documentation Directory (`/docs`)

```
docs/
├── README.md                          # Documentation index (NEW)
├── installation/
│   └── installation-guide.md          # Comprehensive installation guide
├── setup/
│   ├── windows.md                     # Windows-specific setup
│   └── macos.md                       # macOS-specific setup
├── usage/
│   └── user-manual.md                 # Complete user manual
├── guides/
│   ├── quick-start.md                 # Quick start guide
│   └── troubleshooting.md             # Troubleshooting guide
├── development/
│   ├── contributing.md                # Contribution guidelines
│   ├── typescript-patterns.md         # TypeScript patterns
│   └── sidebar-integration.md         # IDE integration
├── planning/
│   ├── todo.md                        # Task list
│   └── optimization-plan.md           # Optimization strategy
├── ai/
│   └── claude-integration.md          # Claude integration
├── CONTEXT_WINDOW_MANAGEMENT.md       # Technical docs (unchanged)
├── DEEP_ANALYSIS_MODE.md              # Technical docs (unchanged)
├── MCP_INTEGRATION.md                 # Technical docs (unchanged)
├── searxng-setup.md                   # Technical docs (unchanged)
└── searxng-troubleshooting.md         # Technical docs (unchanged)
```

### Scripts Directory (`/scripts`)

```
scripts/
├── windows/                           # Windows-specific scripts
│   ├── generate-installer.bat
│   ├── install-dependencies.bat
│   ├── start-application.bat
│   ├── windows-setup.bat
│   └── install.bat
├── install.sh                         # Unix installation script
├── setup-fix.sh                       # Unix setup fix script
├── build.sh                           # Build script (unchanged)
├── cleanup.sh                         # Cleanup script (unchanged)
├── test.sh                            # Test script (unchanged)
└── [other existing scripts]           # Other scripts (unchanged)
```

### Root Directory (Cleaned)

Files remaining in root (approximately 15):
- `README.md` (updated with new links)
- `LICENSE`
- `.gitignore` (updated)
- `.env.example`
- `.eslintrc.json`
- `.prettierrc.json`
- `.prettierignore`
- `package.json`
- `package-lock.json`
- `tsconfig.json`
- `tsconfig.node.json`
- `vite.config.ts`
- `index.html`
- `styles.css`
- `install-wrapper.bat` (backward compatibility)
- `start-wrapper.bat` (backward compatibility)
- `install-symlink.sh` (backward compatibility symlink)
- `setup-fix-symlink.sh` (backward compatibility symlink)

---

## Backward Compatibility

### For Users

**Windows Users:**
- Old batch file references will still work through wrapper scripts
- Wrappers are in root: `install-wrapper.bat`, `start-wrapper.bat`
- Wrappers automatically call scripts in new locations

**Unix/Linux/macOS Users:**
- Symbolic links provide backward compatibility
- `install-symlink.sh` → `scripts/install.sh`
- `setup-fix-symlink.sh` → `scripts/setup-fix.sh`

### For Developers

**Updating Code References:**
```bash
# Old reference
./install.bat

# New reference (preferred)
./scripts/windows/install.bat

# Backward compatible (works but deprecated)
./install-wrapper.bat
```

**Updating Documentation Links:**
```markdown
# Old link
[Installation Guide](INSTALL.md)

# New link
[Installation Guide](docs/installation/installation-guide.md)
```

---

## Updating External References

### If You Have Bookmarks

Update your bookmarks from old paths to new paths using the tables above.

### If You Have External Documentation

1. **Replace root documentation references** with paths to `/docs` subdirectories
2. **Update script paths** to reference `/scripts` or `/scripts/windows`
3. **Use the documentation index** at `docs/README.md` for quick reference

### If You Have Automation Scripts

1. **Review script references** in your automation
2. **Update paths** to new locations in `/scripts` directory
3. **Test thoroughly** to ensure scripts still execute correctly

---

## Testing Your Updates

### Verify Documentation Links

```bash
# Check all markdown files for broken links
find docs -name "*.md" -exec grep -H "\[.*\](.*)" {} \;
```

### Verify Script Execution

**Windows:**
```batch
# Test wrapper scripts
call install-wrapper.bat --help
call start-wrapper.bat --help

# Test direct script access
call scripts\windows\install.bat --help
```

**Unix/Linux/macOS:**
```bash
# Test symlinks
./install-symlink.sh --help
./setup-fix-symlink.sh --help

# Test direct script access
./scripts/install.sh --help
```

---

## Changes Not Affecting Users

The following changes are internal and don't require user action:
- File organization within directories
- Git history is preserved
- No breaking changes to application functionality
- Configuration files remain compatible

---

## Need Help?

### Questions About Migration

- **Documentation**: See [docs/README.md](./README.md) for the new documentation structure
- **Issues**: Report problems on [GitHub Issues](https://github.com/GerdsenAI/GerdsenAI_AutoCoder/issues)
- **Support**: Check the [Troubleshooting Guide](./guides/troubleshooting.md)

### Reporting Broken Links

If you find broken links or references:
1. Note the old path and expected new path
2. Open an issue on GitHub with details
3. Include the file path and line number if possible

---

## Rollback Information

If you need to access old file locations temporarily:

```bash
# View files at old locations in git history
git show main~1:INSTALL.md
git show main~1:CONTRIBUTING.md

# Check out specific version before reorganization
git checkout <commit-before-reorganization>
```

The last commit before reorganization: `[commit-hash-will-be-added]`

---

## Timeline

- **2025-10-30**: Repository reorganization completed
- **2025-10-30**: Migration guide published
- **Future**: Wrapper scripts may be deprecated after 6 months

---

## Summary

This reorganization improves repository structure without breaking existing workflows. All old paths remain accessible through wrapper scripts and symlinks. Update your bookmarks and documentation references at your convenience using this guide.

**Questions?** Open an issue on GitHub or check the documentation index.

---

**Last Updated**: October 30, 2025
