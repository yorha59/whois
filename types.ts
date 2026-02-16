
export enum ServiceType {
  SSH = 'SSH',
  HTTP = 'HTTP',
  HTTPS = 'HTTPS',
  FTP = 'FTP',
  REDIS = 'Redis',
  POSTGRES = 'PostgreSQL',
  MYSQL = 'MySQL',
  UNKNOWN = 'Generic Port'
}

export interface NetworkPort {
  port: number;
  service: ServiceType;
  banner?: string;
}

export interface ScannedHost {
  ip: string;
  hostname?: string;
  status: 'online' | 'offline';
  ports: NetworkPort[];
  lastSeen: string;
}

export interface ScanResult {
  hosts: ScannedHost[];
  totalScanned: number;
  startTime: string;
  endTime?: string;
}
