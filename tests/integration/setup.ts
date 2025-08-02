import { spawn, ChildProcess } from 'child_process';
import { promisify } from 'util';
import { exec as execCallback } from 'child_process';
import fetch from 'node-fetch';

const exec = promisify(execCallback);

interface ServiceConfig {
  name: string;
  healthCheckUrl: string;
  dockerImage?: string;
  command?: string;
  env?: Record<string, string>;
  ports?: Record<string, string>;
  retries?: number;
  retryDelay?: number;
}

export class TestEnvironment {
  private services: Map<string, ChildProcess | string> = new Map();
  private dockerContainers: Set<string> = new Set();

  constructor(private config: {
    services: ServiceConfig[];
    useDocker?: boolean;
  }) {}

  async setup(): Promise<void> {
    console.log('🚀 Setting up test environment...');
    
    for (const service of this.config.services) {
      if (this.config.useDocker && service.dockerImage) {
        await this.startDockerService(service);
      } else if (service.command) {
        await this.startLocalService(service);
      }
      
      await this.waitForService(service);
    }
    
    console.log('✅ Test environment ready');
  }

  async teardown(): Promise<void> {
    console.log('🧹 Cleaning up test environment...');
    
    // Stop local services
    for (const [name, process] of this.services) {
      if (typeof process !== 'string') {
        console.log(`Stopping ${name}...`);
        process.kill();
      }
    }
    
    // Stop Docker containers
    if (this.config.useDocker) {
      for (const containerId of this.dockerContainers) {
        try {
          await exec(`docker stop ${containerId}`);
          await exec(`docker rm ${containerId}`);
        } catch (error) {
          console.warn(`Failed to stop container ${containerId}:`, error);
        }
      }
    }
    
    console.log('✅ Cleanup complete');
  }

  private async startDockerService(service: ServiceConfig): Promise<void> {
    console.log(`🐳 Starting ${service.name} with Docker...`);
    
    // Build docker run command
    let cmd = `docker run -d --name test-${service.name}-${Date.now()}`;
    
    // Add port mappings
    if (service.ports) {
      for (const [host, container] of Object.entries(service.ports)) {
        cmd += ` -p ${host}:${container}`;
      }
    }
    
    // Add environment variables
    if (service.env) {
      for (const [key, value] of Object.entries(service.env)) {
        cmd += ` -e ${key}="${value}"`;
      }
    }
    
    cmd += ` ${service.dockerImage}`;
    
    try {
      const { stdout } = await exec(cmd);
      const containerId = stdout.trim();
      this.dockerContainers.add(containerId);
      this.services.set(service.name, containerId);
      console.log(`✅ ${service.name} started (${containerId.substring(0, 12)})`);
    } catch (error) {
      throw new Error(`Failed to start ${service.name}: ${error}`);
    }
  }

  private async startLocalService(service: ServiceConfig): Promise<void> {
    console.log(`🖥️  Starting ${service.name} locally...`);
    
    const process = spawn(service.command!, {
      shell: true,
      env: { ...process.env, ...service.env },
      detached: false
    });
    
    process.stdout.on('data', (data) => {
      console.log(`[${service.name}] ${data}`);
    });
    
    process.stderr.on('data', (data) => {
      console.error(`[${service.name}] ${data}`);
    });
    
    process.on('error', (error) => {
      console.error(`[${service.name}] Process error:`, error);
    });
    
    this.services.set(service.name, process);
    console.log(`✅ ${service.name} started`);
  }

  private async waitForService(service: ServiceConfig): Promise<void> {
    const retries = service.retries || 30;
    const delay = service.retryDelay || 1000;
    
    console.log(`⏳ Waiting for ${service.name} to be ready...`);
    
    for (let i = 0; i < retries; i++) {
      try {
        const response = await fetch(service.healthCheckUrl);
        if (response.ok) {
          console.log(`✅ ${service.name} is ready`);
          return;
        }
      } catch (error) {
        // Service not ready yet
      }
      
      if (i < retries - 1) {
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
    
    throw new Error(`${service.name} failed to start after ${retries} attempts`);
  }

  async checkHealth(): Promise<Record<string, boolean>> {
    const health: Record<string, boolean> = {};
    
    for (const service of this.config.services) {
      try {
        const response = await fetch(service.healthCheckUrl);
        health[service.name] = response.ok;
      } catch (error) {
        health[service.name] = false;
      }
    }
    
    return health;
  }
}

// Default test environment configuration
export const defaultTestEnv = new TestEnvironment({
  useDocker: process.env.USE_DOCKER === 'true',
  services: [
    {
      name: 'ollama',
      healthCheckUrl: 'http://localhost:11434/api/tags',
      dockerImage: 'ollama/ollama:latest',
      ports: { '11434': '11434' },
      command: 'ollama serve',
      retries: 30,
      retryDelay: 2000
    },
    {
      name: 'searxng',
      healthCheckUrl: 'http://localhost:8080/healthz',
      dockerImage: 'searxng/searxng:latest',
      ports: { '8080': '8080' },
      env: {
        SEARXNG_SECRET: 'test-secret'
      }
    },
    {
      name: 'chromadb',
      healthCheckUrl: 'http://localhost:8000/api/v1/heartbeat',
      dockerImage: 'chromadb/chroma:latest',
      ports: { '8000': '8000' },
      env: {
        IS_PERSISTENT: 'TRUE',
        PERSIST_DIRECTORY: '/chroma/chroma'
      }
    }
  ]
});

// Vitest global setup
export async function setup() {
  if (process.env.SKIP_SERVICE_SETUP !== 'true') {
    await defaultTestEnv.setup();
  }
}

export async function teardown() {
  if (process.env.SKIP_SERVICE_SETUP !== 'true') {
    await defaultTestEnv.teardown();
  }
}