import { useEffect } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { AppLayout } from './components/layout/AppLayout';
import {
  EnvironmentSwitcher,
  getStoredEnvironment,
  PRODUCTION_CONFIRMATION_KEY,
  PRODUCTION_REASON_KEY,
} from './components/EnvironmentSwitcher';
import { DashboardPage } from './pages/DashboardPage';
import { AgentsPage } from './pages/AgentsPage';
import { ConnectionsPage } from './pages/ConnectionsPage';
import { IntegrationsPage } from './pages/IntegrationsPage';
import { IntegrationStudioPage } from './pages/IntegrationStudioPage';
import { DataHubPage } from './pages/DataHubPage';
import { AssetRegistryPage } from './pages/AssetRegistryPage';
import { SettingsPage } from './pages/SettingsPage';
// Operations Center
import { OperationsDashboardPage } from './pages/OperationsDashboardPage';
import { AlertsPage } from './pages/AlertsPage';
import { IncidentsPage } from './pages/IncidentsPage';
import { AutomationPlaybooksListPage } from './pages/AutomationPlaybooksListPage';
import { AutomationPlaybookEditorPage } from './pages/AutomationPlaybookEditorPage';
import { PlaybookRunDetailPage } from './pages/PlaybookRunDetailPage';
// Governance Center
import { GovernanceDashboardPage } from './pages/GovernanceDashboardPage';
import { PoliciesPage } from './pages/PoliciesPage';
import { RulesetsPage } from './pages/RulesetsPage';
import { StandardsPage } from './pages/StandardsPage';
import { ApprovalsPage } from './pages/ApprovalsPage';
import { AuditLogPage } from './pages/AuditLogPage';
// Rationalization Engine
import { RationalizationDashboardPage } from './pages/RationalizationDashboardPage';
import { ApplicationPortfolioPage } from './pages/ApplicationPortfolioPage';
import { ScenariosPage } from './pages/ScenariosPage';
import { PlaybooksPage } from './pages/PlaybooksPage';
import { ProjectsPage } from './pages/ProjectsPage';
// AI Components
import { AIAssistButton } from './components/ai';
// Billing
import { PricingPage } from './pages/PricingPage';

function App() {
  useEffect(() => {
    const nativeFetch = window.fetch.bind(window);

    window.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
      const headers = new Headers(init?.headers);
      headers.set('x-environment', getStoredEnvironment());

      const productionConfirmed = sessionStorage.getItem(PRODUCTION_CONFIRMATION_KEY);
      const changeReason = sessionStorage.getItem(PRODUCTION_REASON_KEY);
      if (productionConfirmed === 'true' && changeReason) {
        headers.set('x-production-confirmed', 'true');
        headers.set('x-change-reason', changeReason);
      }

      return nativeFetch(input, {
        ...init,
        headers,
      });
    };

    return () => {
      window.fetch = nativeFetch;
    };
  }, []);

  return (
    <>
      <EnvironmentSwitcher />
      <Routes>
        <Route path="/" element={<AppLayout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<DashboardPage />} />
          <Route path="agents" element={<AgentsPage />} />
          <Route path="connections" element={<ConnectionsPage />} />
          <Route path="integrations" element={<IntegrationsPage />} />
          <Route path="integrations/:id/edit" element={<IntegrationStudioPage />} />
          <Route path="integrations/new" element={<IntegrationStudioPage />} />
          <Route path="data-hub" element={<DataHubPage />} />
          <Route path="assets" element={<AssetRegistryPage />} />
          {/* Operations Center */}
          <Route path="operations" element={<OperationsDashboardPage />} />
          <Route path="operations/alerts" element={<AlertsPage />} />
          <Route path="operations/incidents" element={<IncidentsPage />} />
          <Route path="operations/playbooks" element={<AutomationPlaybooksListPage />} />
          <Route path="operations/playbooks/new" element={<AutomationPlaybookEditorPage />} />
          <Route path="operations/playbooks/:id/edit" element={<AutomationPlaybookEditorPage />} />
          <Route path="operations/playbooks/:id/runs/:runId" element={<PlaybookRunDetailPage />} />
          {/* Governance Center */}
          <Route path="governance" element={<GovernanceDashboardPage />} />
          <Route path="governance/policies" element={<PoliciesPage />} />
          <Route path="governance/rulesets" element={<RulesetsPage />} />
          <Route path="governance/standards" element={<StandardsPage />} />
          <Route path="governance/approvals" element={<ApprovalsPage />} />
          <Route path="governance/audit" element={<AuditLogPage />} />
          {/* Rationalization Engine */}
          <Route path="rationalization" element={<RationalizationDashboardPage />} />
          <Route path="rationalization/applications" element={<ApplicationPortfolioPage />} />
          <Route path="rationalization/scenarios" element={<ScenariosPage />} />
          <Route path="rationalization/playbooks" element={<PlaybooksPage />} />
          <Route path="rationalization/projects" element={<ProjectsPage />} />
          <Route path="pricing" element={<PricingPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
      </Routes>
      {/* Global AI Assistant Button */}
      <AIAssistButton context="general" />
    </>
  );
}

export default App;
