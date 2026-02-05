import {
  Connector,
  ConnectionConfig,
  HealthCheckResult,
  ReadResult,
  WriteResult,
  Record,
  TaskConfig,
} from './types';

/**
 * Test harness for connector development
 */
export class ConnectorTestHarness {
  private connector: Connector;
  private config: ConnectionConfig;

  constructor(connector: Connector, config: ConnectionConfig) {
    this.connector = connector;
    this.config = config;
  }

  /**
   * Run all standard tests for the connector
   */
  async runAllTests(): Promise<TestReport> {
    const results: TestResult[] = [];

    // Test initialization
    results.push(await this.testInitialization());

    // Test health check
    if (this.connector.metadata.capabilities.supportsHealthCheck) {
      results.push(await this.testHealthCheck());
    }

    // Test discovery
    if (this.connector.metadata.capabilities.supportsDiscover && this.connector.discover) {
      results.push(await this.testDiscovery());
    }

    // Test cleanup
    results.push(await this.testClose());

    return {
      connectorName: this.connector.metadata.name,
      connectorVersion: this.connector.metadata.version,
      timestamp: new Date().toISOString(),
      results,
      passed: results.every((r) => r.passed),
    };
  }

  private async testInitialization(): Promise<TestResult> {
    const start = Date.now();
    try {
      await this.connector.initialize(this.config);
      return {
        name: 'initialization',
        passed: true,
        durationMs: Date.now() - start,
      };
    } catch (error) {
      return {
        name: 'initialization',
        passed: false,
        durationMs: Date.now() - start,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  private async testHealthCheck(): Promise<TestResult> {
    const start = Date.now();
    try {
      const result = await this.connector.healthCheck();
      return {
        name: 'healthCheck',
        passed: result.healthy,
        durationMs: Date.now() - start,
        details: result,
      };
    } catch (error) {
      return {
        name: 'healthCheck',
        passed: false,
        durationMs: Date.now() - start,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  private async testDiscovery(): Promise<TestResult> {
    const start = Date.now();
    try {
      const resources = await this.connector.discover!();
      return {
        name: 'discovery',
        passed: true,
        durationMs: Date.now() - start,
        details: { resourceCount: resources.length, resources: resources.slice(0, 5) },
      };
    } catch (error) {
      return {
        name: 'discovery',
        passed: false,
        durationMs: Date.now() - start,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  private async testClose(): Promise<TestResult> {
    const start = Date.now();
    try {
      await this.connector.close();
      return {
        name: 'close',
        passed: true,
        durationMs: Date.now() - start,
      };
    } catch (error) {
      return {
        name: 'close',
        passed: false,
        durationMs: Date.now() - start,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Test a read operation
   */
  async testRead(taskConfig: TaskConfig): Promise<TestResult> {
    if (!this.connector.read) {
      return {
        name: 'read',
        passed: false,
        durationMs: 0,
        error: 'Connector does not support read operations',
      };
    }

    const start = Date.now();
    try {
      const result = await this.connector.read(taskConfig);
      return {
        name: 'read',
        passed: true,
        durationMs: Date.now() - start,
        details: {
          recordCount: result.data.records.length,
          hasMore: result.data.metadata?.hasMore,
          metrics: result.metrics,
        },
      };
    } catch (error) {
      return {
        name: 'read',
        passed: false,
        durationMs: Date.now() - start,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Test a write operation
   */
  async testWrite(taskConfig: TaskConfig, records: Record[]): Promise<TestResult> {
    if (!this.connector.write) {
      return {
        name: 'write',
        passed: false,
        durationMs: 0,
        error: 'Connector does not support write operations',
      };
    }

    const start = Date.now();
    try {
      const result = await this.connector.write(taskConfig, records);
      return {
        name: 'write',
        passed: result.recordsWritten === records.length,
        durationMs: Date.now() - start,
        details: {
          recordsWritten: result.recordsWritten,
          expectedRecords: records.length,
          metrics: result.metrics,
        },
      };
    } catch (error) {
      return {
        name: 'write',
        passed: false,
        durationMs: Date.now() - start,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }
}

export interface TestResult {
  name: string;
  passed: boolean;
  durationMs: number;
  error?: string;
  details?: unknown;
}

export interface TestReport {
  connectorName: string;
  connectorVersion: string;
  timestamp: string;
  results: TestResult[];
  passed: boolean;
}

/**
 * Print a test report to console
 */
export function printTestReport(report: TestReport): void {
  console.log('\n' + '='.repeat(60));
  console.log(`Connector Test Report: ${report.connectorName} v${report.connectorVersion}`);
  console.log(`Timestamp: ${report.timestamp}`);
  console.log('='.repeat(60) + '\n');

  for (const result of report.results) {
    const status = result.passed ? '✓' : '✗';
    const color = result.passed ? '\x1b[32m' : '\x1b[31m';
    console.log(`${color}${status}\x1b[0m ${result.name} (${result.durationMs}ms)`);
    if (result.error) {
      console.log(`  Error: ${result.error}`);
    }
  }

  console.log('\n' + '-'.repeat(60));
  const passedCount = report.results.filter((r) => r.passed).length;
  console.log(`Results: ${passedCount}/${report.results.length} tests passed`);
  console.log(`Overall: ${report.passed ? '\x1b[32mPASSED\x1b[0m' : '\x1b[31mFAILED\x1b[0m'}`);
  console.log('='.repeat(60) + '\n');
}
