import React from 'react';
import { ScannedHost } from '../types';
import HostCard from './HostCard';

interface DashboardProps {
  hosts: ScannedHost[];
  isScanning: boolean;
  progress: number;
  startScan: () => void;
  stats: { total: number; active: number };
}

const Dashboard: React.FC<DashboardProps> = ({ hosts, isScanning, progress, startScan, stats }) => {
  return (
    <div className="max-w-7xl mx-auto px-4 py-8">
      {/* Header & Controls */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-12">
        <div>
          <h1 className="text-4xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-indigo-500 mb-2">
            Whois
          </h1>
          <p className="text-slate-400">High-performance local network service discovery</p>
        </div>
        
        <div className="flex items-center gap-4">
          <div className="text-right hidden sm:block">
            <div className="text-xs uppercase text-slate-500 font-semibold tracking-wider">Active Hosts</div>
            <div className="text-2xl font-mono text-emerald-400">{stats.active}</div>
          </div>
          <button 
            onClick={startScan}
            disabled={isScanning}
            className={`px-8 py-3 rounded-xl font-semibold transition-all flex items-center gap-3 ${
              isScanning 
                ? 'bg-slate-800 text-slate-500 cursor-not-allowed border border-slate-700' 
                : 'bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-900/20'
            }`}
          >
            {isScanning ? (
              <>
                <i className="fa-solid fa-circle-notch fa-spin"></i>
                Scanning...
              </>
            ) : (
              <>
                <i className="fa-solid fa-radar"></i>
                Start Network Scan
              </>
            )}
          </button>
        </div>
      </div>

      {/* Progress Bar */}
      {isScanning && (
        <div className="mb-12">
          <div className="flex justify-between items-end mb-2">
            <span className="text-sm font-medium text-blue-400">Scan Progress</span>
            <span className="text-sm font-mono text-slate-400">{progress}% ({stats.total}/254 IPs)</span>
          </div>
          <div className="h-2 w-full bg-slate-800 rounded-full overflow-hidden border border-slate-700">
            <div 
              className="h-full bg-blue-500 transition-all duration-300 scanning-animation shadow-[0_0_15px_rgba(59,130,246,0.5)]" 
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>
      )}

      {/* Results Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {hosts.map((host) => (
          <HostCard key={host.ip} host={host} />
        ))}
        
        {!isScanning && hosts.length === 0 && (
          <div className="col-span-full flex flex-col items-center justify-center py-24 bg-slate-900/30 rounded-3xl border-2 border-dashed border-slate-800">
            <i className="fa-solid fa-network-wired text-6xl text-slate-800 mb-6"></i>
            <h3 className="text-xl font-semibold text-slate-500">No hosts detected yet</h3>
            <p className="text-slate-600">Click start scan to explore your local network</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default Dashboard;