import { useState } from 'react';

type FreshnessSLA = 'real_time' | 'hourly' | 'daily' | 'weekly';
type PricingModel = 'free' | 'pay_per_query' | 'subscription' | 'tiered';
type ProductStatus = 'draft' | 'active' | 'deprecated';

interface DataProduct {
  id: string;
  name: string;
  description: string;
  owner: string;
  ownerTeam: string;
  freshnessSla: FreshnessSLA;
  qualityScore: number;
  pricingModel: PricingModel;
  pricePerQuery?: number;
  monthlyPrice?: number;
  status: ProductStatus;
  subscriberCount: number;
  queryCount30d: number;
  revenue30d: number;
  tags: string[];
  lastUpdated: string;
}

const FRESHNESS_LABELS: Record<FreshnessSLA, { label: string; color: string }> = {
  real_time: { label: 'Real-time', color: 'text-green-400 bg-green-900/30' },
  hourly: { label: 'Hourly', color: 'text-blue-400 bg-blue-900/30' },
  daily: { label: 'Daily', color: 'text-yellow-400 bg-yellow-900/30' },
  weekly: { label: 'Weekly', color: 'text-gray-400 bg-gray-700' },
};

const MOCK_PRODUCTS: DataProduct[] = [
  { id: 'dp1', name: 'Customer 360 Profile', description: 'Unified customer profile combining CRM, support, billing, and product usage data. Enriched with behavioral segments and propensity scores.', owner: 'Sarah Chen', ownerTeam: 'Customer Data Platform', freshnessSla: 'hourly', qualityScore: 96.5, pricingModel: 'subscription', monthlyPrice: 500, status: 'active', subscriberCount: 14, queryCount30d: 45600, revenue30d: 7000, tags: ['customer', 'profile', 'segmentation'], lastUpdated: '2 hours ago' },
  { id: 'dp2', name: 'Revenue Pipeline Metrics', description: 'Real-time sales pipeline data with stage progression, conversion rates, and forecast accuracy. Sourced from Salesforce and Stripe.', owner: 'Mike Torres', ownerTeam: 'Revenue Ops', freshnessSla: 'real_time', qualityScore: 98.2, pricingModel: 'pay_per_query', pricePerQuery: 0.005, status: 'active', subscriberCount: 8, queryCount30d: 128000, revenue30d: 640, tags: ['revenue', 'pipeline', 'forecast'], lastUpdated: '30 sec ago' },
  { id: 'dp3', name: 'Product Usage Analytics', description: 'Feature-level usage analytics with user cohorts, retention curves, and engagement scores. 90-day rolling window.', owner: 'Alex Kim', ownerTeam: 'Product Analytics', freshnessSla: 'daily', qualityScore: 94.8, pricingModel: 'free', status: 'active', subscriberCount: 22, queryCount30d: 89000, revenue30d: 0, tags: ['product', 'usage', 'analytics'], lastUpdated: '6 hours ago' },
  { id: 'dp4', name: 'Compliance Audit Dataset', description: 'Pre-compiled compliance evidence including access logs, policy evaluations, data lineage traces, and governance decisions.', owner: 'Jennifer Wu', ownerTeam: 'GRC', freshnessSla: 'daily', qualityScore: 99.1, pricingModel: 'subscription', monthlyPrice: 1000, status: 'active', subscriberCount: 5, queryCount30d: 2400, revenue30d: 5000, tags: ['compliance', 'audit', 'governance'], lastUpdated: '12 hours ago' },
  { id: 'dp5', name: 'Supply Chain Signal Feed', description: 'Aggregated supply chain signals from IoT sensors, ERP systems, and logistics partners. Includes anomaly detection overlays.', owner: 'Raj Patel', ownerTeam: 'Supply Chain Ops', freshnessSla: 'real_time', qualityScore: 92.3, pricingModel: 'tiered', monthlyPrice: 250, status: 'active', subscriberCount: 7, queryCount30d: 340000, revenue30d: 1750, tags: ['supply-chain', 'iot', 'logistics'], lastUpdated: '15 sec ago' },
  { id: 'dp6', name: 'Employee Experience Index', description: 'Aggregated employee engagement metrics from HRIS, survey platforms, and collaboration tools.', owner: 'Lisa Park', ownerTeam: 'People Analytics', freshnessSla: 'weekly', qualityScore: 88.7, pricingModel: 'free', status: 'draft', subscriberCount: 0, queryCount30d: 0, revenue30d: 0, tags: ['hr', 'engagement', 'people'], lastUpdated: '3 days ago' },
];

export function DataProductsPage() {
  const [selectedStatus, setSelectedStatus] = useState<string>('all');
  const [selectedPricing, setSelectedPricing] = useState<string>('all');

  const filtered = MOCK_PRODUCTS
    .filter(p => selectedStatus === 'all' || p.status === selectedStatus)
    .filter(p => selectedPricing === 'all' || p.pricingModel === selectedPricing);

  const totalRevenue = MOCK_PRODUCTS.reduce((s, p) => s + p.revenue30d, 0);
  const totalQueries = MOCK_PRODUCTS.reduce((s, p) => s + p.queryCount30d, 0);
  const totalSubscribers = MOCK_PRODUCTS.reduce((s, p) => s + p.subscriberCount, 0);
  const avgQuality = MOCK_PRODUCTS.filter(p => p.status === 'active').reduce((s, p) => s + p.qualityScore, 0) / MOCK_PRODUCTS.filter(p => p.status === 'active').length;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Data Products</h1>
          <p className="text-sm text-gray-400 mt-1">Discover, subscribe to, and publish governed data products across your organization</p>
        </div>
        <button className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium transition-colors">
          Create Data Product
        </button>
      </div>

      {/* Summary Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {[
          { label: 'Active Products', value: MOCK_PRODUCTS.filter(p => p.status === 'active').length, detail: `of ${MOCK_PRODUCTS.length} total` },
          { label: 'Total Subscribers', value: totalSubscribers, detail: 'across all products' },
          { label: 'Queries (30d)', value: totalQueries.toLocaleString(), detail: 'total data queries' },
          { label: 'Revenue (30d)', value: `$${totalRevenue.toLocaleString()}`, detail: `avg quality: ${avgQuality.toFixed(1)}%` },
        ].map(stat => (
          <div key={stat.label} className="bg-gray-800/50 border border-gray-700 rounded-xl p-4">
            <p className="text-xs text-gray-500 uppercase tracking-wider">{stat.label}</p>
            <p className="text-2xl font-bold text-white mt-1">{stat.value}</p>
            <p className="text-xs text-gray-500 mt-0.5">{stat.detail}</p>
          </div>
        ))}
      </div>

      {/* Filters */}
      <div className="flex gap-3">
        <select value={selectedStatus} onChange={e => setSelectedStatus(e.target.value)} className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-300 text-sm">
          <option value="all">All Statuses</option>
          <option value="active">Active</option>
          <option value="draft">Draft</option>
          <option value="deprecated">Deprecated</option>
        </select>
        <select value={selectedPricing} onChange={e => setSelectedPricing(e.target.value)} className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-300 text-sm">
          <option value="all">All Pricing</option>
          <option value="free">Free</option>
          <option value="pay_per_query">Pay Per Query</option>
          <option value="subscription">Subscription</option>
          <option value="tiered">Tiered</option>
        </select>
      </div>

      {/* Product Cards */}
      <div className="space-y-3">
        {filtered.map(product => {
          const freshness = FRESHNESS_LABELS[product.freshnessSla];
          return (
            <div key={product.id} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5 hover:border-gray-600 transition-colors cursor-pointer">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-1">
                    <h3 className="text-white font-semibold">{product.name}</h3>
                    <span className={`text-xs px-2 py-0.5 rounded ${freshness.color}`}>{freshness.label}</span>
                    {product.status === 'draft' && <span className="text-xs px-2 py-0.5 rounded bg-gray-700 text-gray-400">Draft</span>}
                    {product.status === 'deprecated' && <span className="text-xs px-2 py-0.5 rounded bg-red-900/30 text-red-400">Deprecated</span>}
                  </div>
                  <p className="text-xs text-gray-500 mb-2">by {product.owner} ({product.ownerTeam}) &middot; Updated {product.lastUpdated}</p>
                  <p className="text-sm text-gray-400 mb-3">{product.description}</p>
                  <div className="flex flex-wrap gap-1.5">
                    {product.tags.map(tag => (
                      <span key={tag} className="text-xs px-2 py-0.5 bg-gray-700/50 text-gray-400 rounded">{tag}</span>
                    ))}
                  </div>
                </div>
                <div className="flex items-center gap-6 ml-6 text-sm shrink-0">
                  <div className="text-right">
                    <p className="text-gray-500 text-xs">Quality</p>
                    <p className={`font-mono ${product.qualityScore >= 95 ? 'text-green-400' : product.qualityScore >= 90 ? 'text-yellow-400' : 'text-red-400'}`}>
                      {product.qualityScore}%
                    </p>
                  </div>
                  <div className="text-right">
                    <p className="text-gray-500 text-xs">Subscribers</p>
                    <p className="text-gray-300 font-mono">{product.subscriberCount}</p>
                  </div>
                  <div className="text-right">
                    <p className="text-gray-500 text-xs">Queries (30d)</p>
                    <p className="text-gray-300 font-mono">{product.queryCount30d.toLocaleString()}</p>
                  </div>
                  <div className="text-right">
                    <p className="text-gray-500 text-xs">Pricing</p>
                    <p className="text-gray-300 text-xs">
                      {product.pricingModel === 'free' ? 'Free' :
                       product.pricingModel === 'pay_per_query' ? `$${product.pricePerQuery}/query` :
                       product.pricingModel === 'subscription' ? `$${product.monthlyPrice}/mo` :
                       `From $${product.monthlyPrice}/mo`}
                    </p>
                  </div>
                  <button className="px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-xs font-medium transition-colors">
                    {product.status === 'active' ? 'Subscribe' : 'View'}
                  </button>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
