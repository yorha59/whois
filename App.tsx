
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

// W-1: Detect subnet via Rust backend (Tauri) or fallback
const detectLocalSubnet = async (): Promise<{ subnet: string; localIp: string }> => {
  // In Tauri: call Rust backend to detect real network
  if (tauriInvoke) {
    try {
      const info = await tauriInvoke('detect_network');
      return { subnet: info.subnet, localIp: info.local_ip };
    } catch (e) {
      console.error("detect_network failed:", e);
    }
  }
  // Browser fallback: try to detect from hostname, else default
  try {
    const hostname = window.location.hostname;
    if (hostname && hostname.match(/^\d+\.\d+\.\d+\.\d+$/)) {
      const parts = hostname.split('.');
      return { subnet: `${parts[0]}.${parts[1]}.${parts[2]}`, localIp: hostname };
    }
  } catch (e) {}
  return { subnet: "192.168.1", localIp: "unknown" };
};

const App: React.FC = () => {
  const [hosts, setHosts] = useState<ScannedHost[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState(0);
  const [scanStats, setScanStats] = useState({ total: 0, active: 0 });
  const [networkInfo, setNetworkInfo] = useState<{ subnet: string; localIp: string } | null>(null);

  // Auto-detect network on mount
  useEffect(() => {
    detectLocalSubnet().then(info => setNetworkInfo(info));
  }, []);

  const startScan = useCallback(async () => {
    setIsScanning(true);
    setProgress(0);
    setHosts([]);
    setScanStats({ total: 0, active: 0 });

    const net = networkInfo || await detectLocalSubnet();

    // REAL SCAN LOGIC (TAURI)
    if (tauriInvoke) {
      try {
        setProgress(10); // Show we're starting
        const results: any[] = await tauriInvoke('perform_real_scan', { 
          subnet: net.subnet,
          extraPorts: null  // W-5: can pass custom ports later
        });
        const mappedHosts: ScannedHost[] = results.map(r => ({
          ip: r.ip,
          hostname: r.hostname || undefined,
          status: 'online',
          ports: r.ports.map((p: any) => ({
            port: p.port,
            service: p.service,
            service_label: p.service_label || p.service,
          })) as NetworkPort[],
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

      const ip = `${net.subnet}.${currentIpIdx}`;
      setScanStats(prev => ({ ...prev, total: currentIpIdx }));

      if (Math.random() > 0.9) {
        const foundPorts = [22, 80, 443, 6379, 3306, 5432, 8080]
          .filter(() => Math.random() > 0.7)
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
  }, [networkInfo]);

  return (
    <div className="min-h-screen">
      {networkInfo && (
        <div style={{ 
          position: 'fixed', bottom: 12, left: 12, 
          background: 'rgba(0,0,0,0.6)', color: '#8892b0',
          padding: '4px 10px', borderRadius: 6, fontSize: 11,
          fontFamily: 'JetBrains Mono, monospace', zIndex: 100
        }}>
          {tauriInvoke ? '🟢' : '🟡'} {networkInfo.localIp} / {networkInfo.subnet}.0/24
        </div>
      )}
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
