# FangaOS Documentation Index

Welcome to the FangaOS documentation! This index will help you find the information you need.

## 📚 Documentation Overview

### For Users & Contributors

| Document | Description | Best For |
|----------|-------------|----------|
| [README.md](../README.md) | Project overview, features, and build instructions | First-time visitors |
| [QUICKSTART.md](QUICKSTART.md) | 5-minute setup guide | Getting started quickly |
| [ROADMAP.md](../ROADMAP.md) | Development plan with features and priorities | Understanding project direction |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | How to contribute to the project | New contributors |
| [CHANGELOG.md](../CHANGELOG.md) | Version history and changes | Tracking updates |

### For Developers

| Document | Description | Best For |
|----------|-------------|----------|
| [DEVELOPMENT.md](DEVELOPMENT.md) | Developer guide with tips and workflows | Active developers |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System design and architecture details | Understanding internals |
| [IDEAS.md](IDEAS.md) | Brainstorming and experimental features | Innovation and planning |

### GitHub Templates

| Template | Purpose |
|----------|---------|
| [Bug Report](.github/ISSUE_TEMPLATE/bug_report.md) | Report bugs and issues |
| [Feature Request](.github/ISSUE_TEMPLATE/feature_request.md) | Suggest new features |
| [Documentation](.github/ISSUE_TEMPLATE/documentation.md) | Report documentation issues |

## 🎯 Quick Navigation

### I want to...

**...learn about the project**
→ Start with [README.md](../README.md)

**...set up my development environment**
→ Follow [QUICKSTART.md](QUICKSTART.md)

**...understand the codebase**
→ Read [ARCHITECTURE.md](ARCHITECTURE.md)

**...contribute code**
→ Check [CONTRIBUTING.md](../CONTRIBUTING.md) and [DEVELOPMENT.md](DEVELOPMENT.md)

**...see what's planned**
→ Browse [ROADMAP.md](../ROADMAP.md)

**...propose a new feature**
→ Review [IDEAS.md](IDEAS.md) and [ROADMAP.md](../ROADMAP.md) first

**...report a bug**
→ Use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md)

**...find good first issues**
→ Check GitHub issues labeled `good-first-issue`

## 📖 Reading Order

### For New Users
1. [README.md](../README.md) - Overview
2. [QUICKSTART.md](QUICKSTART.md) - Setup
3. [ROADMAP.md](../ROADMAP.md) - Future plans

### For New Contributors
1. [README.md](../README.md) - Overview
2. [QUICKSTART.md](QUICKSTART.md) - Setup
3. [CONTRIBUTING.md](../CONTRIBUTING.md) - Guidelines
4. [DEVELOPMENT.md](DEVELOPMENT.md) - Developer guide
5. [ARCHITECTURE.md](ARCHITECTURE.md) - System design

### For Active Developers
1. [DEVELOPMENT.md](DEVELOPMENT.md) - Daily reference
2. [ARCHITECTURE.md](ARCHITECTURE.md) - Design decisions
3. [ROADMAP.md](../ROADMAP.md) - What to work on
4. [IDEAS.md](IDEAS.md) - Future possibilities

## 📝 Document Summaries

### README.md
**What**: Project introduction and getting started
**Contains**:
- Overview and features
- Build requirements
- Quick build instructions
- Project structure
- Contact information

### QUICKSTART.md
**What**: Fast setup guide for new users
**Contains**:
- Step-by-step installation
- Common commands
- Troubleshooting
- Next steps

### ROADMAP.md
**What**: Comprehensive development plan
**Contains**:
- Current status
- Short-term goals (1-3 months)
- Medium-term goals (3-9 months)
- Long-term goals (9+ months)
- Feature priorities
- Version milestones

### CONTRIBUTING.md
**What**: Contribution guidelines
**Contains**:
- How to report bugs
- How to suggest features
- Code style guidelines
- Pull request process
- Commit message format

### CHANGELOG.md
**What**: Version history
**Contains**:
- Release notes
- Breaking changes
- New features per version
- Bug fixes

### DEVELOPMENT.md
**What**: Developer workflow guide
**Contains**:
- Development setup
- Building and testing
- Debugging techniques
- Common tasks
- Tips and tricks

### ARCHITECTURE.md
**What**: System design documentation
**Contains**:
- Design principles
- System architecture
- Memory layout
- Boot process
- Module organization
- Future architecture plans

### IDEAS.md
**What**: Brainstorming and experimental features
**Contains**:
- Quick win features
- Innovative ideas
- Future enhancements
- Experimental concepts
- Implementation priorities

## 🔧 Technical Documentation

### Build System
- **Makefile**: Build automation for creating ISOs and running QEMU
- **kernel/Cargo.toml**: Rust workspace configuration
- **kernel/x86_64-fanga-kernel.json**: Custom Rust target specification
- **kernel/linker.ld**: Linker script for kernel layout

### Configuration
- **limine.conf**: Bootloader configuration
- **.gitignore**: Files to ignore in git
- **rust-toolchain.toml**: Rust toolchain specification

### CI/CD
- **.github/workflows/ci.yml**: Continuous integration workflow

## 🎓 Learning Path

### Beginner (New to OS Development)
1. Read [README.md](../README.md)
2. Follow [QUICKSTART.md](QUICKSTART.md)
3. Browse kernel code in `kernel/crates/fanga-kernel/src/main.rs`
4. Make small changes and test
5. Read [OSDev Wiki](https://wiki.osdev.org/) for OS concepts

### Intermediate (Some OS Knowledge)
1. Study [ARCHITECTURE.md](ARCHITECTURE.md)
2. Understand interrupt handling in `kernel/crates/fanga-arch-x86_64/src/interrupts/`
3. Read [ROADMAP.md](../ROADMAP.md) for feature ideas
4. Pick a "good first issue"
5. Follow [CONTRIBUTING.md](../CONTRIBUTING.md) to submit

### Advanced (Ready to Lead)
1. Design new subsystems using [ARCHITECTURE.md](ARCHITECTURE.md) as guide
2. Propose features based on [IDEAS.md](IDEAS.md)
3. Review PRs from other contributors
4. Help maintain documentation
5. Guide new contributors

## 🔍 Finding Information

### Search Tips
- **GitHub search**: Use repository search for code
- **grep/ripgrep**: Search documentation files locally
- **GitHub issues**: Search for related discussions

### Key Terms
- **HHDM**: Higher Half Direct Map
- **IDT**: Interrupt Descriptor Table
- **PIC**: Programmable Interrupt Controller
- **Limine**: Bootloader used by FangaOS
- **QEMU**: Emulator for testing

## 📮 Getting Help

### Documentation Issues
If you find documentation that is:
- Unclear or confusing
- Outdated or incorrect
- Missing important information

Please open an issue using the [Documentation template](.github/ISSUE_TEMPLATE/documentation.md)

### Questions
- **General questions**: Open a GitHub Discussion
- **Bug reports**: Use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md)
- **Feature ideas**: Use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md)

## 🔄 Keeping Updated

This documentation is actively maintained. To stay up to date:
- **Watch the repository** on GitHub for notifications
- **Check CHANGELOG.md** for version updates
- **Review ROADMAP.md** periodically for plan changes
- **Read commit messages** for detailed changes

## 📊 Documentation Statistics

- **Total documents**: 8 main documents + 3 templates
- **Total words**: ~25,000+ words
- **Coverage**: Architecture, development, contribution, roadmap
- **Languages**: English (open to translations!)

## 🤝 Contributing to Documentation

Documentation contributions are highly valued! You can:
- Fix typos and grammar
- Clarify confusing sections
- Add examples and diagrams
- Translate to other languages
- Keep information up to date

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

**Last Updated**: December 2024  
**Maintained by**: FangaOS Team

For questions about this documentation index, please open an issue on GitHub.
