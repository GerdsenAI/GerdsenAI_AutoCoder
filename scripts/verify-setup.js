#!/usr/bin/env node

/**
 * GerdsenAI Socrates - Setup Verification Script
 * Validates all required dependencies and services for a smooth user experience
 */

import { spawn, exec } from 'child_process';
import { promisify } from 'util';
import fs from 'fs';
import path from 'path';

const execAsync = promisify(exec);

// ANSI color codes for better output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m'
};

const log = {
  info: (msg) => console.log(`${colors.blue}ℹ${colors.reset} ${msg}`),
  success: (msg) => console.log(`${colors.green}✅${colors.reset} ${msg}`),
  warning: (msg) => console.log(`${colors.yellow}⚠️${colors.reset} ${msg}`),
  error: (msg) => console.log(`${colors.red}❌${colors.reset} ${msg}`),
  header: (msg) => console.log(`\n${colors.bright}${colors.cyan}🚀 ${msg}${colors.reset}`),
  subheader: (msg) => console.log(`\n${colors.bright}${msg}${colors.reset}`)
};

// Configuration for required services
const requiredServices = {
  ollama: {
    name: 'Ollama',
    port: 11434,
    healthEndpoint: 'http://localhost:11434/api/tags',
    installUrl: 'https://ollama.ai/download',
    description: 'AI model server (required for core functionality)',
    critical: true
  },
  searxng: {
    name: 'SearXNG',
    port: 8080,
    healthEndpoint: 'http://localhost:8080/healthz',
    installUrl: 'https://docs.searxng.org/admin/installation.html',
    description: 'Web search service (optional, enhances search features)',
    critical: false
  },
  chromadb: {
    name: 'ChromaDB',
    port: 8000,
    healthEndpoint: 'http://localhost:8000/api/v1/heartbeat',
    installUrl: 'https://docs.trychroma.com/getting-started',
    description: 'Vector database for document storage (required for RAG features)',
    critical: true
  }
};

class SetupVerifier {
  constructor() {
    this.results = {
      platform: { status: 'unknown', details: null },
      nodejs: { status: 'unknown', details: null },
      rust: { status: 'unknown', details: null },
      dependencies: { status: 'unknown', details: null },
      services: {}
    };
    
    this.issues = [];
    this.suggestions = [];
  }

  async checkPlatform() {
    log.subheader('Checking Platform Environment');
    
    try {
      const platform = process.platform;
      let architecture = process.arch;
      let details = { platform, architecture };
      
      if (platform === 'darwin') {
        // Get more detailed macOS info
        try {
          const { stdout: osVersion } = await execAsync('sw_vers -productVersion');
          const { stdout: archInfo } = await execAsync('uname -m');
          const { stdout: cpuInfo } = await execAsync('sysctl -n machdep.cpu.brand_string');
          
          details = {
            platform: 'macOS',
            version: osVersion.trim(),
            architecture: archInfo.trim(),
            cpu: cpuInfo.trim(),
            isAppleSilicon: archInfo.trim() === 'arm64'
          };
          
          log.success(`macOS ${details.version} (${details.isAppleSilicon ? 'Apple Silicon' : 'Intel'})`);
          
          if (details.isAppleSilicon) {
            log.info('✨ Apple Silicon detected - native performance available');
          } else {
            log.info('🖥️ Intel Mac detected - full compatibility');
          }
          
        } catch (error) {
          log.warning('Could not get detailed macOS info');
        }
      } else if (platform === 'win32') {
        details.platform = 'Windows';
        log.success(`Windows (${architecture})`);
      } else if (platform === 'linux') {
        details.platform = 'Linux';
        log.success(`Linux (${architecture})`);
      } else {
        log.warning(`Unsupported platform: ${platform}`);
      }
      
      this.results.platform = { status: 'good', details };
      
    } catch (error) {
      log.error('Failed to detect platform information');
      this.results.platform = { status: 'error', details: { error: error.message } };
    }
  }

  async checkNodeJS() {
    log.subheader('Checking Node.js Environment');
    
    try {
      const { stdout } = await execAsync('node --version');
      const version = stdout.trim();
      const majorVersion = parseInt(version.slice(1).split('.')[0]);
      
      if (majorVersion >= 20) {
        log.success(`Node.js ${version} (compatible)`);
        this.results.nodejs = { status: 'good', details: { version, compatible: true } };
      } else {
        log.warning(`Node.js ${version} (requires 20.19+ for Vite 7)`);
        this.results.nodejs = { status: 'warning', details: { version, compatible: false } };
        this.issues.push('Node.js version may be too old for Vite 7');
        this.suggestions.push('Update Node.js to version 20.19+ or 22.12+');
      }
    } catch (error) {
      log.error('Node.js not found');
      this.results.nodejs = { status: 'error', details: { error: error.message } };
      this.issues.push('Node.js is not installed');
      this.suggestions.push('Install Node.js from https://nodejs.org/');
    }
  }

  async checkRust() {
    log.subheader('Checking Rust Environment');
    
    try {
      const { stdout } = await execAsync('rustc --version');
      const version = stdout.trim();
      log.success(`Rust ${version}`);
      this.results.rust = { status: 'good', details: { version } };
    } catch (error) {
      log.error('Rust not found');
      this.results.rust = { status: 'error', details: { error: error.message } };
      this.issues.push('Rust is not installed');
      this.suggestions.push('Install Rust from https://rustup.rs/');
    }
  }

  async checkDependencies() {
    log.subheader('Checking Project Dependencies');
    
    try {
      // Check if node_modules exists
      const nodeModulesPath = path.join(process.cwd(), 'node_modules');
      if (!fs.existsSync(nodeModulesPath)) {
        log.warning('Node modules not installed');
        this.results.dependencies = { status: 'warning', details: { installed: false } };
        this.suggestions.push('Run "npm install" to install dependencies');
        return;
      }

      // Check if package-lock.json exists
      const packageLockPath = path.join(process.cwd(), 'package-lock.json');
      if (!fs.existsSync(packageLockPath)) {
        log.warning('package-lock.json missing');
        this.suggestions.push('Run "npm install" to generate package-lock.json');
      }

      // Check Tauri dependencies
      const tauriSrcPath = path.join(process.cwd(), 'src-tauri', 'Cargo.lock');
      if (!fs.existsSync(tauriSrcPath)) {
        log.warning('Tauri dependencies not built');
        this.suggestions.push('Run "npm run tauri build" to build Tauri dependencies');
      }

      log.success('Dependencies appear to be installed');
      this.results.dependencies = { status: 'good', details: { installed: true } };

    } catch (error) {
      log.error('Failed to check dependencies');
      this.results.dependencies = { status: 'error', details: { error: error.message } };
      this.issues.push('Cannot verify project dependencies');
    }
  }

  async checkService(serviceKey, config) {
    log.info(`Checking ${config.name} on port ${config.port}...`);
    
    try {
      const response = await fetch(config.healthEndpoint, {
        method: 'GET',
        timeout: 5000
      });
      
      if (response.ok) {
        log.success(`${config.name} is running and healthy`);
        this.results.services[serviceKey] = { 
          status: 'good', 
          details: { running: true, port: config.port } 
        };
      } else {
        log.warning(`${config.name} responded with status ${response.status}`);
        this.results.services[serviceKey] = { 
          status: 'warning', 
          details: { running: true, port: config.port, status: response.status } 
        };
        this.suggestions.push(`Check ${config.name} configuration`);
      }
    } catch (error) {
      if (config.critical) {
        log.error(`${config.name} is not accessible (${error.message})`);
        this.results.services[serviceKey] = { 
          status: 'error', 
          details: { running: false, port: config.port, error: error.message } 
        };
        this.issues.push(`${config.name} is required but not running`);
        this.suggestions.push(`Install and start ${config.name}: ${config.installUrl}`);
      } else {
        log.warning(`${config.name} is not accessible (optional service)`);
        this.results.services[serviceKey] = { 
          status: 'warning', 
          details: { running: false, port: config.port, optional: true } 
        };
        this.suggestions.push(`Optional: Install ${config.name} for enhanced features: ${config.installUrl}`);
      }
    }
  }

  async checkAllServices() {
    log.subheader('Checking External Services');
    
    for (const [serviceKey, config] of Object.entries(requiredServices)) {
      await this.checkService(serviceKey, config);
    }
  }

  async checkPortAvailability() {
    log.subheader('Checking Port Availability');
    
    const developmentPorts = [3000, 3001, 5173, 1420]; // Common dev ports
    
    for (const port of developmentPorts) {
      try {
        const { stdout } = await execAsync(`lsof -ti:${port} 2>/dev/null || netstat -an | grep :${port} 2>/dev/null || true`);
        if (stdout.trim()) {
          log.warning(`Port ${port} is in use`);
          this.suggestions.push(`Port ${port} may conflict with development server`);
        } else {
          log.success(`Port ${port} is available`);
        }
      } catch (error) {
        // Port checking failed, but not critical
        log.info(`Could not check port ${port} availability`);
      }
    }
  }

  generateReport() {
    log.header('Setup Verification Report');
    
    console.log('\n📊 Summary:');
    const totalChecks = Object.keys(this.results).length + Object.keys(this.results.services).length;
    const goodChecks = Object.values(this.results).filter(r => r.status === 'good').length + 
                      Object.values(this.results.services).filter(r => r.status === 'good').length;
    
    console.log(`   ${goodChecks}/${totalChecks} checks passed`);
    
    if (this.issues.length > 0) {
      console.log('\n🚨 Critical Issues:');
      this.issues.forEach(issue => log.error(issue));
    }
    
    if (this.suggestions.length > 0) {
      console.log('\n💡 Suggestions:');
      this.suggestions.forEach(suggestion => log.info(suggestion));
    }
    
    // Overall status
    const criticalIssues = this.issues.length;
    const warnings = Object.values(this.results).filter(r => r.status === 'warning').length +
                    Object.values(this.results.services).filter(r => r.status === 'warning').length;
    
    console.log('\n🎯 Overall Status:');
    if (criticalIssues === 0) {
      if (warnings === 0) {
        log.success('All systems ready! 🎉');
        console.log('   You can start development with: npm run dev');
      } else {
        log.warning('Ready with minor issues');
        console.log('   You can start development, but some features may be limited');
      }
    } else {
      log.error('Setup incomplete');
      console.log('   Please resolve critical issues before starting development');
    }
    
    return {
      ready: criticalIssues === 0,
      criticalIssues,
      warnings,
      results: this.results
    };
  }

  async createSetupScript() {
    log.subheader('Generating Setup Commands');
    
    const setupCommands = [];
    
    if (this.results.nodejs.status === 'error') {
      setupCommands.push('# Install Node.js');
      setupCommands.push('curl -fsSL https://nodejs.org/dist/v20.15.0/node-v20.15.0-linux-x64.tar.xz | tar -xJ');
    }
    
    if (this.results.rust.status === 'error') {
      setupCommands.push('# Install Rust');
      setupCommands.push('curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh');
    }
    
    if (this.results.dependencies.status !== 'good') {
      setupCommands.push('# Install project dependencies');
      setupCommands.push('npm install');
    }
    
    // Service installation commands
    Object.entries(this.results.services).forEach(([serviceKey, result]) => {
      if (result.status === 'error') {
        const config = requiredServices[serviceKey];
        setupCommands.push(`# Install ${config.name}`);
        setupCommands.push(`# Visit: ${config.installUrl}`);
      }
    });
    
    if (setupCommands.length > 0) {
      const scriptPath = path.join(process.cwd(), 'setup-fix.sh');
      fs.writeFileSync(scriptPath, setupCommands.join('\n') + '\n');
      log.success(`Generated setup script: ${scriptPath}`);
    }
  }
}

// Main execution
async function main() {
  log.header('GerdsenAI Socrates - Setup Verification');
  console.log('Checking system requirements and dependencies...\n');
  
  const verifier = new SetupVerifier();
  
  try {
    await verifier.checkPlatform();
    await verifier.checkNodeJS();
    await verifier.checkRust();
    await verifier.checkDependencies();
    await verifier.checkAllServices();
    await verifier.checkPortAvailability();
    
    const report = verifier.generateReport();
    await verifier.createSetupScript();
    
    // Save detailed report for debugging
    const reportPath = path.join(process.cwd(), 'setup-verification-report.json');
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    log.info(`Detailed report saved to: ${reportPath}`);
    
    process.exit(report.ready ? 0 : 1);
    
  } catch (error) {
    log.error(`Verification failed: ${error.message}`);
    process.exit(1);
  }
}

// Handle command line arguments
const args = process.argv.slice(2);
if (args.includes('--help') || args.includes('-h')) {
  console.log(`
🚀 GerdsenAI Socrates Setup Verification

Usage:
  node scripts/verify-setup.js [options]

Options:
  --help, -h     Show this help message
  
This script checks:
  • Node.js and Rust installations
  • Project dependencies
  • Required services (Ollama, ChromaDB, SearXNG)
  • Port availability for development

The script will generate:
  • setup-fix.sh - Commands to fix identified issues
  • setup-verification-report.json - Detailed results

Exit codes:
  0 - System ready for development
  1 - Critical issues found
`);
  process.exit(0);
}

main();