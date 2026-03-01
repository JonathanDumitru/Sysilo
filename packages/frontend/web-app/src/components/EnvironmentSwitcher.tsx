import { useEffect, useMemo, useState } from 'react';

export type AppEnvironment = 'dev' | 'staging' | 'prod';

export const ENVIRONMENT_STORAGE_KEY = 'sysilo:selected-environment';
export const ENVIRONMENT_EVENT = 'sysilo:environment-changed';
export const PRODUCTION_CONFIRMATION_KEY = 'sysilo:production-confirmed';
export const PRODUCTION_REASON_KEY = 'sysilo:production-reason';

const ENVIRONMENTS: AppEnvironment[] = ['dev', 'staging', 'prod'];

export function getStoredEnvironment(): AppEnvironment {
  const value = localStorage.getItem(ENVIRONMENT_STORAGE_KEY);
  return value === 'staging' || value === 'prod' ? value : 'dev';
}

export function setStoredEnvironment(environment: AppEnvironment): void {
  localStorage.setItem(ENVIRONMENT_STORAGE_KEY, environment);
  window.dispatchEvent(new CustomEvent<AppEnvironment>(ENVIRONMENT_EVENT, { detail: environment }));
}

export function EnvironmentSwitcher() {
  const [environment, setEnvironment] = useState<AppEnvironment>(() => getStoredEnvironment());

  useEffect(() => {
    const onEnvironmentChanged = (event: Event) => {
      const value = (event as CustomEvent<AppEnvironment>).detail;
      if (value) {
        setEnvironment(value);
      }
    };
    window.addEventListener(ENVIRONMENT_EVENT, onEnvironmentChanged);
    return () => window.removeEventListener(ENVIRONMENT_EVENT, onEnvironmentChanged);
  }, []);

  const badgeClassName = useMemo(() => {
    if (environment === 'prod') return 'bg-red-100 text-red-700 border-red-200';
    if (environment === 'staging') return 'bg-amber-100 text-amber-700 border-amber-200';
    return 'bg-emerald-100 text-emerald-700 border-emerald-200';
  }, [environment]);

  return (
    <div className="fixed right-6 top-20 z-40 flex items-center gap-2 rounded-lg border border-gray-200 bg-white/95 px-3 py-2 shadow-sm backdrop-blur">
      <label htmlFor="environment-switcher" className="text-xs font-medium uppercase tracking-wide text-gray-500">
        Environment
      </label>
      <select
        id="environment-switcher"
        value={environment}
        onChange={(event) => {
          const nextEnvironment = event.target.value as AppEnvironment;
          setEnvironment(nextEnvironment);
          setStoredEnvironment(nextEnvironment);
        }}
        className="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm text-gray-800 outline-none focus:border-primary-500"
      >
        {ENVIRONMENTS.map((value) => (
          <option key={value} value={value}>
            {value}
          </option>
        ))}
      </select>
      <span className={`rounded-full border px-2 py-0.5 text-xs font-semibold uppercase ${badgeClassName}`}>
        {environment}
      </span>
    </div>
  );
}
