import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';
import fs from 'fs';

/**
 * Load environment variables from .env file
 * This ensures .env variables are available during config time
 */
function loadDotEnv(envPath = '.env') {
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

// https://vitejs.dev/config/
export default defineConfig(({ command, mode }) => {
  // Load environment variables
  const env = { ...process.env, ...loadDotEnv() };
  
  // Port configuration
  const preferredPort = parseInt(env.VITE_DEV_PORT) || 3000;
  const autoPortDetection = env.AUTO_PORT_DETECTION !== 'false';
  const host = env.DEV_HOST || 'localhost';
  const debug = env.PORT_DEBUG === 'true';
  
  if (debug) {
    console.log('🔧 Vite config - Port settings:', {
      preferredPort,
      autoPortDetection,
      host,
      mode,
      command
    });
  }

  return {
    plugins: [react()],
    
    // Explicitly define the entry point
    build: {
      rollupOptions: {
        input: {
          main: resolve(__dirname, 'index.html'),
        },
      },
    },
    
    // Explicitly include dependencies for pre-bundling
    optimizeDeps: {
      include: [
        'react', 
        'react-dom', 
        '@tauri-apps/api',
        'react-syntax-highlighter',
        'react-markdown'
      ],
    },

    // Dynamic server configuration for Tauri
    server: {
      port: preferredPort,
      strictPort: !autoPortDetection, // Allow Vite to find alternative ports if auto-detection is enabled
      host: host,
      // Enable CORS for Tauri
      cors: true,
      // Improve HMR for development
      hmr: {
        port: preferredPort + 1000, // Use a different port for HMR to avoid conflicts
      },
    },

    // Resolve paths
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src'),
      },
    },
    
    // Environment variable handling
    define: {
      // Make port info available to the frontend if needed
      __DEV_PORT__: JSON.stringify(preferredPort),
      __AUTO_PORT_DETECTION__: JSON.stringify(autoPortDetection),
    },
  };
});
