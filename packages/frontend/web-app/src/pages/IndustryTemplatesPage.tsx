import { useState } from 'react';

type Vertical = 'healthcare' | 'finance' | 'manufacturing' | 'retail_ecommerce' | 'government';

interface Template {
  id: string;
  name: string;
  vertical: Vertical;
  description: string;
  keyConnectors: string[];
  complianceFrameworks: string[];
  estimatedSetupMinutes: number;
  installCount: number;
  avgRating: number;
  pricingTier: 'free' | 'professional' | 'enterprise';
}

const VERTICAL_META: Record<Vertical, { label: string; color: string; icon: string }> = {
  healthcare: { label: 'Healthcare', color: 'text-emerald-400 bg-emerald-900/30 border-emerald-700', icon: 'H' },
  finance: { label: 'Finance', color: 'text-blue-400 bg-blue-900/30 border-blue-700', icon: 'F' },
  manufacturing: { label: 'Manufacturing', color: 'text-orange-400 bg-orange-900/30 border-orange-700', icon: 'M' },
  retail_ecommerce: { label: 'Retail & E-commerce', color: 'text-purple-400 bg-purple-900/30 border-purple-700', icon: 'R' },
  government: { label: 'Government', color: 'text-red-400 bg-red-900/30 border-red-700', icon: 'G' },
};

const MOCK_TEMPLATES: Template[] = [
  {
    id: '1', name: 'Healthcare HIPAA Compliance Suite', vertical: 'healthcare',
    description: 'HIPAA governance policies, patient data lineage, HL7/FHIR integration playbooks, PHI tagging rules, and clinical data quality monitoring. Deploy a fully compliant healthcare integration environment in under an hour.',
    keyConnectors: ['Epic', 'Cerner', 'HL7/FHIR', 'Lab Systems'],
    complianceFrameworks: ['HIPAA', 'HITECH'],
    estimatedSetupMinutes: 45, installCount: 890, avgRating: 4.8, pricingTier: 'professional',
  },
  {
    id: '2', name: 'Financial Services SOX & Risk Suite', vertical: 'finance',
    description: 'SOX compliance workflows, fraud signal routing, risk engine integration, audit-ready dashboards, and trade surveillance pipelines. Built for banks, insurers, and fintech companies.',
    keyConnectors: ['Bloomberg', 'Plaid', 'Core Banking', 'SWIFT'],
    complianceFrameworks: ['SOX', 'PCI-DSS', 'GDPR'],
    estimatedSetupMinutes: 60, installCount: 1240, avgRating: 4.7, pricingTier: 'enterprise',
  },
  {
    id: '3', name: 'Manufacturing IoT & Supply Chain Suite', vertical: 'manufacturing',
    description: 'IoT sensor ingestion pipelines, supply chain DAGs, quality control automation, ERP sync, and predictive maintenance models. Connects shop floor to cloud seamlessly.',
    keyConnectors: ['SAP', 'Siemens MindSphere', 'OPC-UA', 'MQTT'],
    complianceFrameworks: ['ISO 9001', 'ISO 27001'],
    estimatedSetupMinutes: 50, installCount: 670, avgRating: 4.6, pricingTier: 'professional',
  },
  {
    id: '4', name: 'Retail Customer 360 Suite', vertical: 'retail_ecommerce',
    description: 'Inventory sync, order routing, marketplace connectors, customer 360 data model, and recommendation engine pipelines. Unify your commerce operations.',
    keyConnectors: ['Shopify', 'Amazon SP-API', 'Salesforce Commerce', 'Stripe'],
    complianceFrameworks: ['PCI-DSS', 'GDPR', 'CCPA'],
    estimatedSetupMinutes: 40, installCount: 1580, avgRating: 4.5, pricingTier: 'professional',
  },
  {
    id: '5', name: 'Government FedRAMP & Data Residency Suite', vertical: 'government',
    description: 'FedRAMP-aligned governance, data residency enforcement, citizen data masking, FOIA response automation, and legacy mainframe connectors. Built for public sector compliance.',
    keyConnectors: ['Mainframe CICS', 'SFTP', 'GovCloud S3', 'Active Directory'],
    complianceFrameworks: ['FedRAMP', 'FISMA', 'NIST 800-53'],
    estimatedSetupMinutes: 90, installCount: 340, avgRating: 4.9, pricingTier: 'enterprise',
  },
];

export function IndustryTemplatesPage() {
  const [selectedVertical, setSelectedVertical] = useState<string>('all');
  const verticals: (Vertical | 'all')[] = ['all', 'healthcare', 'finance', 'manufacturing', 'retail_ecommerce', 'government'];

  const filtered = MOCK_TEMPLATES.filter(t => selectedVertical === 'all' || t.vertical === selectedVertical);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Industry Solution Templates</h1>
        <p className="text-sm text-gray-400 mt-1">Pre-built, one-click deployable integration packages tailored to your industry. Configure 80% instantly — customize the last 20%.</p>
      </div>

      {/* Vertical Tabs */}
      <div className="flex gap-2 overflow-x-auto pb-2">
        {verticals.map(v => {
          const meta = v === 'all' ? null : VERTICAL_META[v];
          const isActive = selectedVertical === v;
          return (
            <button
              key={v}
              onClick={() => setSelectedVertical(v)}
              className={`px-4 py-2 rounded-lg text-sm font-medium whitespace-nowrap transition-colors border ${
                isActive
                  ? (meta ? meta.color : 'text-white bg-gray-700 border-gray-600')
                  : 'text-gray-400 bg-gray-800/50 border-gray-700 hover:border-gray-600'
              }`}
            >
              {v === 'all' ? 'All Verticals' : meta!.label}
            </button>
          );
        })}
      </div>

      {/* Template Cards */}
      <div className="space-y-4">
        {filtered.map(template => {
          const meta = VERTICAL_META[template.vertical];
          return (
            <div key={template.id} className="bg-gray-800/50 border border-gray-700 rounded-xl p-6 hover:border-gray-600 transition-colors">
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-4">
                  <div className={`w-12 h-12 rounded-xl flex items-center justify-center text-lg font-bold border ${meta.color}`}>
                    {meta.icon}
                  </div>
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-1">
                      <h3 className="text-lg font-semibold text-white">{template.name}</h3>
                      <span className={`text-xs px-2 py-0.5 rounded-full border ${meta.color}`}>{meta.label}</span>
                    </div>
                    <p className="text-sm text-gray-400 mb-4 max-w-3xl">{template.description}</p>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                      <div>
                        <p className="text-gray-500 text-xs mb-1.5 uppercase tracking-wider">Key Connectors</p>
                        <div className="flex flex-wrap gap-1.5">
                          {template.keyConnectors.map(c => (
                            <span key={c} className="px-2 py-0.5 bg-gray-700 text-gray-300 rounded text-xs">{c}</span>
                          ))}
                        </div>
                      </div>
                      <div>
                        <p className="text-gray-500 text-xs mb-1.5 uppercase tracking-wider">Compliance</p>
                        <div className="flex flex-wrap gap-1.5">
                          {template.complianceFrameworks.map(f => (
                            <span key={f} className="px-2 py-0.5 bg-blue-900/30 text-blue-300 rounded text-xs border border-blue-800">{f}</span>
                          ))}
                        </div>
                      </div>
                      <div>
                        <p className="text-gray-500 text-xs mb-1.5 uppercase tracking-wider">Stats</p>
                        <div className="flex items-center gap-3 text-xs text-gray-400">
                          <span>{template.installCount.toLocaleString()} deployments</span>
                          <span className="text-yellow-400">{template.avgRating} rating</span>
                          <span>{template.estimatedSetupMinutes} min setup</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <button className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium transition-colors shrink-0 ml-4">
                  Deploy Template
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
