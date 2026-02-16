
import React, { useState, useEffect, useCallback } from 'react';
import { ScannedHost, ServiceType, NetworkPort } from './types';
import { COMMON_PORTS } from './constants';
import Dashboard from './components/Dashboard';

// Attempt to import Tauri's invoke if running inside Tauri
let tauriInvoke: any = null;
try {
  // @ts-ignore
  if (window.__TAURI__) {
    import('@tauri-apps/api/tauri').then(mod => {
      tauriInvoke = mod.invoke;
    });
  }
} catch (e) {
  console.log("Not running in Tauri environment.");
}

// Detect local subnet from hostname or default to common subnet
const detectLocalSubnet = async (): Promise<string> => {
  try {
    // Try to get local network info
    const hostname = window.location.hostname;
    if (hostname && hostname.match(/^\d+\.\d+\.\d+\.\d+$/)) {
      const parts = hostname.split('.');
      return `${parts[0]}.${parts[1]}.${parts[2]}`;
    }
  } catch (e) {
    console.log("Could not detect subnet, using default");
  }
  // Default to 192.168.1 for most home networks
  return "192.168.1";
};

const App: React.FC = () => {
  const [hosts, setHosts] = useState<ScannedHost[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState(0);
  const [scanStats, setScanStats] = useState({ total: 0, active: 0 });

  const startScan = useCallback(async () => {
    setIsScanning(true);
    setProgress(0);
    setHosts([]);
    setScanStats({ total: 0, active: 0 });

    // REAL SCAN LOGIC (TAURI)
    if (tauriInvoke) {
      try {
        // Auto-detect subnet from local IP
        const localSubnet = await detectLocalSubnet();
        const results: any[] = await tauriInvoke('perform_real_scan', { subnet: localSubnet });
        const mappedHosts: ScannedHost[] = results.map(r => ({
          ip: r.ip,
          hostname: r.hostname || undefined,
          status: 'online',
          ports: r.ports as NetworkPort[],
          lastSeen: new Date().toLocaleTimeString()
        }));
        setHosts(mappedHosts);
        setScanStats({ total: 254, active: mappedHosts.length });
        setProgress(100);
        setIsScanning(false);
        return;
      } catch (err) {
        console.error("Tauri scan failed, falling back to simulation", err);
      }
    }

    // SIMULATION LOGIC (BROWSER FALLBACK)
    const totalIps = 254;
    let currentIpIdx = 1;
    const mdnsNames = ['macbook-pro', 'epson-printer', 'home-assistant', 'raspberrypi', 'smart-hub'];

    const interval = setInterval(() => {
      if (currentIpIdx > totalIps) {
        clearInterval(interval);
        setIsScanning(false);
        return;
      }

      const ip = `192.168.1.${currentIpIdx}`;
      setScanStats(prev => ({ ...prev, total: currentIpIdx }));

      if (Math.random() > 0.9) {
        const foundPorts = [22, 80, 443, 6379]
          .filter(() => Math.random() > 0.8)
          .map(port => ({
            port,
            service: COMMON_PORTS[port] || ServiceType.UNKNOWN,
          }));

        if (foundPorts.length > 0 || Math.random() > 0.97) {
          let hostname = Math.random() > 0.5 ? `${mdnsNames[Math.floor(Math.random() * mdnsNames.length)]}.local` : undefined;
          const newHost: ScannedHost = {
            ip,
            status: 'online',
            ports: foundPorts,
            lastSeen: new Date().toLocaleTimeString(),
            hostname
          };
          setHosts(prev => [newHost, ...prev]);
          setScanStats(prev => ({ ...prev, active: prev.active + 1 }));
        }
      }
      setProgress(Math.round((currentIpIdx / totalIps) * 100));
      currentIpIdx++;
    }, 30);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen">
      <Dashboard
        hosts={hosts}
        isScanning={isScanning}
        progress={progress}
        startScan={startScan}
        stats={scanStats}
      />
    </div>
  );
};

export default App;
