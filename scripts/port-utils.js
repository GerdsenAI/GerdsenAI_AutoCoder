#!/usr/bin/env node

/**
 * Port Utility Functions for CSE-Icon AutoCoder
 * Handles dynamic port detection and management
 */

import net from 'net';
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

/**
 * Check if a port is available
 * @param {number} port - Port number to check
 * @param {string} host - Host to check (default: 'localhost')
 * @returns {Promise<boolean>} - True if port is available
 */
export function isPortAvailable(port, host = 'localhost') {
  return new Promise((resolve) => {
    const server = net.createServer();
    
    server.listen(port, host, () => {
      server.once('close', () => {
        resolve(true);
      });
      server.close();
    });
    
    server.on('error', () => {
      resolve(false);
    });
  });
}

/**
 * Find the next available port in a range
 * @param {number} startPort - Starting port number
 * @param {number} endPort - Ending port number (optional, defaults to startPort + 100)
 * @param {string} host - Host to check
 * @returns {Promise<number|null>} - Available port number or null
 */
export async function findAvailablePort(startPort, endPort = startPort + 100, host = 'localhost') {
  for (let port = startPort; port <= endPort; port++) {
    if (await isPortAvailable(port, host)) {
      return port;
    }
  }
  return null;
}

/**
 * Parse port range from string (e.g., "3000-3010")
 * @param {string} rangeStr - Port range string
 * @returns {Object} - {start, end} port numbers
 */
export function parsePortRange(rangeStr) {
  if (!rangeStr || typeof rangeStr !== 'string') {
    return { start: 3000, end: 3010 };
  }
  
  if (rangeStr.includes('-')) {
    const [start, end] = rangeStr.split('-').map(p => parseInt(p.trim()));
    return { start: start || 3000, end: end || 3010 };
  }
  
  const port = parseInt(rangeStr);
  return { start: port || 3000, end: (port || 3000) + 10 };
}

/**
 * Get process using a specific port (Windows-specific)
 * @param {number} port - Port number
 * @returns {string|null} - Process information or null
 */
export function getProcessUsingPort(port) {
  try {
    const result = execSync(`netstat -ano | findstr :${port}`, { encoding: 'utf8' });
    return result.trim();
  } catch (error) {
    return null;
  }
}

/**
 * Kill process using a specific port (Windows-specific)
 * @param {number} port - Port number
 * @returns {boolean} - Success status
 */
export function killProcessUsingPort(port) {
  try {
    const processInfo = getProcessUsingPort(port);
    if (!processInfo) return false;
    
    // Extract PID from netstat output
    const lines = processInfo.split('\n');
    for (const line of lines) {
      const parts = line.trim().split(/\s+/);
      if (parts.length >= 5 && parts[1].includes(`:${port}`)) {
        const pid = parts[4];
        if (pid && pid !== '0') {
          execSync(`taskkill /F /PID ${pid}`, { stdio: 'ignore' });
          console.log(`✅ Killed process ${pid} using port ${port}`);
          return true;
        }
      }
    }
    return false;
  } catch (error) {
    console.warn(`⚠️ Could not kill process on port ${port}:`, error.message);
    return false;
  }
}

/**
 * Load environment variables from .env file
 * @param {string} envPath - Path to .env file
 * @returns {Object} - Environment variables object
 */
export function loadEnvFile(envPath = '.env') {
  const env = {};
  
  if (!fs.existsSync(envPath)) {
    return env;
  }
  
  const content = fs.readFileSync(envPath, 'utf8');
  const lines = content.split('\n');
  
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed && !trimmed.startsWith('#')) {
      const [key, ...valueParts] = trimmed.split('=');
      if (key && valueParts.length > 0) {
        env[key.trim()] = valueParts.join('=').trim();
      }
    }
  }
  
  return env;
}

/**
 * Get port configuration from environment variables
 * @returns {Object} - Port configuration object
 */
export function getPortConfig() {
  const env = { ...process.env, ...loadEnvFile() };
  
  const preferredPort = parseInt(env.VITE_DEV_PORT) || 3000;
  const portRange = parsePortRange(env.VITE_PORT_RANGE);
  const autoDetection = env.AUTO_PORT_DETECTION !== 'false';
  const debug = env.PORT_DEBUG === 'true';
  const maxRetries = parseInt(env.MAX_PORT_RETRIES) || 10;
  const host = env.DEV_HOST || 'localhost';
  
  return {
    preferredPort,
    portRange,
    autoDetection,
    debug,
    maxRetries,
    host
  };
}

/**
 * Find and reserve a port for development
 * @returns {Promise<number>} - Available port number
 */
export async function findDevPort() {
  const config = getPortConfig();
  
  if (config.debug) {
    console.log('🔍 Port detection configuration:', config);
  }
  
  // First, try the preferred port
  if (await isPortAvailable(config.preferredPort, config.host)) {
    if (config.debug) {
      console.log(`✅ Preferred port ${config.preferredPort} is available`);
    }
    return config.preferredPort;
  }
  
  if (config.debug) {
    console.log(`⚠️ Preferred port ${config.preferredPort} is busy`);
  }
  
  // If auto-detection is disabled, fail
  if (!config.autoDetection) {
    throw new Error(`Port ${config.preferredPort} is not available and auto-detection is disabled`);
  }
  
  // Try to find an available port in the range
  const availablePort = await findAvailablePort(
    config.portRange.start,
    config.portRange.end,
    config.host
  );
  
  if (!availablePort) {
    throw new Error(`No available ports found in range ${config.portRange.start}-${config.portRange.end}`);
  }
  
  if (config.debug) {
    console.log(`✅ Found available port: ${availablePort}`);
  }
  
  return availablePort;
}

/**
 * Update Tauri configuration with the detected port
 * @param {number} port - Port number to use
 * @returns {Promise<void>}
 */
export async function updateTauriConfig(port) {
  const configPath = path.join('src-tauri', 'tauri.conf.json');
  
  if (!fs.existsSync(configPath)) {
    console.warn('⚠️ Tauri config file not found, skipping update');
    return;
  }
  
  try {
    const configData = fs.readFileSync(configPath, 'utf8');
    const config = JSON.parse(configData);
    
    // Update the devUrl
    if (config.build) {
      config.build.devUrl = `http://localhost:${port}`;
      
      // Write the updated config
      fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
      console.log(`✅ Updated Tauri config to use port ${port}`);
    }
  } catch (error) {
    console.warn('⚠️ Could not update Tauri config:', error.message);
  }
}

// CLI Interface
if (import.meta.url === `file://${process.argv[1]}`) {
  const command = process.argv[2];
  
  switch (command) {
    case 'find':
      findDevPort()
        .then(port => {
          console.log(port);
          process.exit(0);
        })
        .catch(error => {
          console.error('❌ Error:', error.message);
          process.exit(1);
        });
      break;
      
    case 'check':
      const port = parseInt(process.argv[3]);
      if (!port) {
        console.error('❌ Please provide a port number');
        process.exit(1);
      }
      isPortAvailable(port)
        .then(available => {
          console.log(available ? 'available' : 'busy');
          process.exit(0);
        })
        .catch(error => {
          console.error('❌ Error:', error.message);
          process.exit(1);
        });
      break;
      
    case 'kill':
      const killPort = parseInt(process.argv[3]);
      if (!killPort) {
        console.error('❌ Please provide a port number');
        process.exit(1);
      }
      const success = killProcessUsingPort(killPort);
      process.exit(success ? 0 : 1);
      break;
      
    case 'info':
      const infoPort = parseInt(process.argv[3]);
      if (!infoPort) {
        console.error('❌ Please provide a port number');
        process.exit(1);
      }
      const processInfo = getProcessUsingPort(infoPort);
      if (processInfo) {
        console.log(`Port ${infoPort} is used by:`);
        console.log(processInfo);
      } else {
        console.log(`Port ${infoPort} is available`);
      }
      break;
      
    default:
      console.log(`
CSE-Icon AutoCoder Port Utilities

Usage:
  node scripts/port-utils.js find                 Find available port
  node scripts/port-utils.js check <port>         Check if port is available
  node scripts/port-utils.js kill <port>          Kill process using port
  node scripts/port-utils.js info <port>          Show process info for port

Examples:
  node scripts/port-utils.js find                 # Find next available port
  node scripts/port-utils.js check 3000           # Check if port 3000 is free
  node scripts/port-utils.js kill 3000            # Kill process on port 3000
  node scripts/port-utils.js info 3000            # Show what's using port 3000
`);
      break;
  }
}