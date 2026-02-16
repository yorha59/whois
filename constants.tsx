
import React from 'react';
import { ServiceType } from './types';

export const SERVICE_ICONS: Record<ServiceType, React.ReactNode> = {
  [ServiceType.SSH]: <i className="fa-solid fa-terminal text-blue-400"></i>,
  [ServiceType.HTTP]: <i className="fa-solid fa-globe text-emerald-400"></i>,
  [ServiceType.HTTPS]: <i className="fa-solid fa-lock text-emerald-500"></i>,
  [ServiceType.FTP]: <i className="fa-solid fa-file-transfer text-orange-400"></i>,
  [ServiceType.REDIS]: <i className="fa-solid fa-database text-red-500"></i>,
  [ServiceType.POSTGRES]: <i className="fa-solid fa-database text-indigo-400"></i>,
  [ServiceType.MYSQL]: <i className="fa-solid fa-database text-sky-400"></i>,
  [ServiceType.UNKNOWN]: <i className="fa-solid fa-plug text-slate-400"></i>,
};

export const COMMON_PORTS: Record<number, ServiceType> = {
  21: ServiceType.FTP,
  22: ServiceType.SSH,
  80: ServiceType.HTTP,
  443: ServiceType.HTTPS,
  3306: ServiceType.MYSQL,
  5432: ServiceType.POSTGRES,
  6379: ServiceType.REDIS,
};
