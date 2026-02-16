
import React, { useState } from 'react';
import { ScannedHost } from '../types';
import { SERVICE_ICONS } from '../constants';
import { analyzeSecurity } from '../services/geminiService';

interface HostCardProps {
  host: ScannedHost;
}

const HostCard: React.FC<HostCardProps> = ({ host }) => {
  const [analysis, setAnalysis] = useState<string | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);

  const handleAnalyze = async () => {
    setIsAnalyzing(true);
    const result = await analyzeSecurity(host);
    setAnalysis(result);
    setIsAnalyzing(false);
  };

  const isMDNS = host.hostname?.toLowerCase().endsWith('.local');

  return (
    <div className="bg-slate-900/50 backdrop-blur-sm border border-slate-800 rounded-2xl overflow-hidden hover:border-slate-700 transition-all group flex flex-col h-full">
      <div className="p-6 flex-grow">
        {/* Host Identity */}
        <div className="flex justify-between items-start mb-6">
          <div className="flex-grow">
            <div className="flex items-center gap-2 mb-1">
              <span className="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.6)]"></span>
              <h3 className="text-xl font-bold font-mono text-slate-100">{host.ip}</h3>
            </div>
            <div className="flex flex-wrap items-center gap-2">
              <p className="text-sm font-medium text-slate-300 truncate max-w-[180px]">
                {host.hostname || 'Anonymous Host'}
              </p>
              {isMDNS && (
                <span className="px-1.5 py-0.5 rounded text-[9px] font-bold bg-purple-500/10 text-purple-400 border border-purple-500/20 uppercase tracking-tighter">
                  mDNS
                </span>
              )}
            </div>
            <p className="text-[10px] text-slate-500 uppercase mt-1">
              Last seen {host.lastSeen}
            </p>
          </div>
          <button 
            onClick={handleAnalyze}
            disabled={isAnalyzing}
            className="p-2 text-slate-400 hover:text-blue-400 transition-colors bg-slate-800/50 rounded-lg border border-slate-700 ml-2"
            title="AI Security Insight"
          >
            {isAnalyzing ? <i className="fa-solid fa-spinner fa-spin"></i> : <i className="fa-solid fa-shield-halved"></i>}
          </button>
        </div>

        {/* Services List */}
        <div className="space-y-3">
          <div className="text-[10px] font-bold text-slate-600 uppercase tracking-widest mb-2 flex items-center gap-2">
            <i className="fa-solid fa-list-check opacity-50"></i>
            Detected Services
          </div>
          <div className="grid grid-cols-2 gap-2">
            {host.ports.length > 0 ? (
              host.ports.map((p) => (
                <div key={p.port} className="flex items-center gap-2.5 bg-slate-950/50 p-2.5 rounded-xl border border-slate-800/50 hover:bg-slate-800/30 transition-colors">
                  <div className="text-lg opacity-90">
                    {SERVICE_ICONS[p.service]}
                  </div>
                  <div className="overflow-hidden">
                    <div className="text-[10px] font-mono text-slate-400 leading-none mb-1">:{p.port}</div>
                    <div className="text-[11px] font-semibold text-slate-200 truncate">{p.service}</div>
                  </div>
                </div>
              ))
            ) : (
              <div className="col-span-2 py-2 text-center text-[11px] text-slate-600 italic">
                No open ports detected
              </div>
            )}
          </div>
        </div>

        {/* AI Analysis Result */}
        {analysis && (
          <div className="mt-6 p-4 bg-blue-900/10 border border-blue-900/20 rounded-xl animate-in fade-in slide-in-from-top-2">
            <div className="flex items-center gap-2 text-blue-400 text-xs font-bold mb-2 uppercase tracking-wider">
              <i className="fa-solid fa-sparkles text-[10px]"></i>
              AI Security Analysis
            </div>
            <p className="text-xs text-slate-400 leading-relaxed italic">
              "{analysis}"
            </p>
          </div>
        )}
      </div>
      
      {/* Footer Decoration */}
      <div className="h-1 w-full bg-gradient-to-r from-transparent via-slate-800 to-transparent opacity-30"></div>
    </div>
  );
};

export default HostCard;
