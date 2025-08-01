#!/usr/bin/env node

/**
 * Development Server Startup Script with Dynamic Port Management
 * Handles port detection, Tauri config updates, and coordinated startup
 */

import { spawn } from 'child_process';
import { findDevPort, updateTauriConfig, getPortConfig } from './port-utils.js';
import path from 'path';
import fs from 'fs';

// Parse command line arguments
const args = process.argv.slice(2);
const isPortMode = args.includes('--port') || args.includes('-p');
const isTauriMode = args.includes('--tauri') || args.includes('-t');
const helpMode = args.includes('--help') || args.includes('-h');
const verboseMode = args.includes('--verbose') || args.includes('-v');

// Port argument parsing
let customPort = null;
const portIndex = args.findIndex(arg => arg === '--port' || arg === '-p');
if (portIndex !== -1 && args[portIndex + 1]) {
  customPort = parseInt(args[portIndex + 1]);
}

/**
 * Display help information
 */
function showHelp() {
  console.log(`
🚀 CSE-Icon AutoCoder Development Server

Usage:
  npm run dev                           Start Vite dev server with auto port detection
  npm run dev:port [port]               Start with specific port
  npm run tauri:dev                     Start full Tauri development environment
  
Script Options:
  --port [number]      Use specific port (overrides .env settings)
  --tauri              Start Tauri development mode
  --verbose            Enable verbose logging
  --help               Show this help message

Port Management:
  npm run port:find                     Find next available port
  npm run port:check [port]             Check if port is available
  npm run port:kill [port]              Kill process using port
  npm run port:info [port]              Show process info for port

Environment Configuration:
  Edit .env file to customize port settings and ranges
  See .env.example for all available options

Examples:
  npm run dev                           # Auto-detect port, start Vite only
  npm run dev -- --port 3000           # Force port 3000
  npm run tauri:dev                     # Start full Tauri app with auto port
  npm run tauri:dev -- --verbose       # Start with detailed logging
`);
}

/**
 * Update environment variables for the current process
 */
function updateProcessEnv(port) {
  process.env.VITE_DEV_PORT = port.toString();
  process.env.TAURI_DEV_URL = `http://localhost:${port}`;
  
  if (verboseMode) {
    console.log('📝 Updated environment:', {
      VITE_DEV_PORT: process.env.VITE_DEV_PORT,
      TAURI_DEV_URL: process.env.TAURI_DEV_URL
    });
  }
}

/**
 * Start Vite development server
 */
function startVite(port) {
  console.log(`🔥 Starting Vite development server on port ${port}...`);
  
  const viteProcess = spawn('npx', ['vite', '--port', port.toString()], {
    stdio: 'inherit',
    shell: false, // Fixed DEP0190: removed shell: true for security
    env: { ...process.env, VITE_DEV_PORT: port.toString() }
  });
  
  viteProcess.on('error', (error) => {
    console.error('❌ Failed to start Vite:', error.message);
    process.exit(1);
  });
  
  viteProcess.on('exit', (code) => {
    if (code !== 0) {
      console.error(`❌ Vite exited with code ${code}`);
      process.exit(code);
    }
  });
  
  return viteProcess;
}

/**
 * Start Tauri development environment
 */
function startTauri(port) {
  console.log(`🦀 Starting Tauri development environment with frontend on port ${port}...`);
  
  const tauriProcess = spawn('npx', ['tauri', 'dev'], {
    stdio: 'inherit',
    shell: false, // Fixed DEP0190: removed shell: true for security
    env: { ...process.env, VITE_DEV_PORT: port.toString() }
  });
  
  tauriProcess.on('error', (error) => {
    console.error('❌ Failed to start Tauri:', error.message);
    process.exit(1);
  });
  
  tauriProcess.on('exit', (code) => {
    if (code !== 0) {
      console.error(`❌ Tauri exited with code ${code}`);
      process.exit(code);
    }
  });
  
  return tauriProcess;
}

/**
 * Main startup sequence
 */
async function main() {
  if (helpMode) {
    showHelp();
    return;
  }
  
  console.log('🚀 CSE-Icon AutoCoder Development Startup');
  console.log('==========================================');
  
  try {
    // Determine the port to use
    let selectedPort;
    
    if (customPort) {
      console.log(`📍 Using custom port: ${customPort}`);
      selectedPort = customPort;
    } else {
      console.log('🔍 Detecting available port...');
      selectedPort = await findDevPort();
    }
    
    console.log(`✅ Selected port: ${selectedPort}`);
    
    // Update environment variables
    updateProcessEnv(selectedPort);
    
    // Update Tauri configuration if in Tauri mode
    if (isTauriMode) {
      console.log('🔧 Updating Tauri configuration...');
      await updateTauriConfig(selectedPort);
    }
    
    // Display configuration summary
    const config = getPortConfig();
    if (verboseMode) {
      console.log('📊 Configuration Summary:');
      console.log(`   Port: ${selectedPort}`);
      console.log(`   Host: ${config.host}`);
      console.log(`   Auto-detection: ${config.autoDetection}`);
      console.log(`   Port range: ${config.portRange.start}-${config.portRange.end}`);
      console.log(`   Mode: ${isTauriMode ? 'Tauri Development' : 'Vite Only'}`);
      console.log();
    }
    
    // Start the appropriate development server
    if (isTauriMode) {
      await startTauri(selectedPort);
    } else {
      await startVite(selectedPort);
    }
    
  } catch (error) {
    console.error('❌ Startup failed:', error.message);
    
    // Provide helpful suggestions
    if (error.message.includes('port')) {
      console.log('\n💡 Suggestions:');
      console.log('   • Try a different port: npm run dev -- --port 3001');
      console.log('   • Kill processes using your port: npm run port:kill 3000');
      console.log('   • Check what\'s using the port: npm run port:info 3000');
      console.log('   • Update your .env file with a different port range');
    }
    
    process.exit(1);
  }
}

// Handle process cleanup
process.on('SIGINT', () => {
  console.log('\n🛑 Shutting down development server...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.log('\n🛑 Development server terminated');
  process.exit(0);
});

// Start the application
main().catch(error => {
  console.error('💥 Unexpected error:', error);
  process.exit(1);
});