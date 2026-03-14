import { useState } from 'react';

type ListingTier = 'free' | 'verified' | 'premium';
type ConnectorCategory = 'database' | 'saas' | 'api' | 'messaging' | 'file' | 'iot' | 'erp';

interface MarketplaceListing {
  id: string;
  name: string;
  description: string;
  category: ConnectorCategory;
  tier: ListingTier;
  author: string;
  version: string;
  installCount: number;
  avgRating: number;
  ratingCount: number;
  pricing: 'free' | 'usage_based' | 'flat_rate';
  monthlyPrice?: number;
  tags: string[];
}

const MOCK_LISTINGS: MarketplaceListing[] = [
  { id: '1', name: 'Snowflake Enterprise', description: 'Production-grade Snowflake connector with schema discovery, incremental loading, and time-travel queries', category: 'database', tier: 'verified', author: 'Sysilo', version: '2.4.1', installCount: 3420, avgRating: 4.8, ratingCount: 156, pricing: 'free', tags: ['data-warehouse', 'analytics'] },
  { id: '2', name: 'Salesforce CRM Pro', description: 'Full Salesforce API coverage with bulk operations, CDC streaming, and metadata sync', category: 'saas', tier: 'premium', author: 'CloudBridge Labs', version: '3.1.0', installCount: 2890, avgRating: 4.7, ratingCount: 203, pricing: 'usage_based', monthlyPrice: 0.002, tags: ['crm', 'sales'] },
  { id: '3', name: 'Apache Kafka', description: 'High-throughput Kafka connector with exactly-once semantics and schema registry integration', category: 'messaging', tier: 'verified', author: 'Sysilo', version: '1.8.3', installCount: 4100, avgRating: 4.9, ratingCount: 89, pricing: 'free', tags: ['streaming', 'event-driven'] },
  { id: '4', name: 'SAP S/4HANA', description: 'Enterprise SAP connector with BAPI/RFC, IDoc, and OData support', category: 'erp', tier: 'premium', author: 'EnterpriseTech', version: '2.0.1', installCount: 1650, avgRating: 4.5, ratingCount: 67, pricing: 'flat_rate', monthlyPrice: 299, tags: ['erp', 'manufacturing'] },
  { id: '5', name: 'REST API Universal', description: 'Generic REST API connector with OAuth2, pagination, and response transformation', category: 'api', tier: 'free', author: 'Community', version: '1.2.0', installCount: 8900, avgRating: 4.3, ratingCount: 312, pricing: 'free', tags: ['rest', 'universal'] },
  { id: '6', name: 'MongoDB Atlas', description: 'MongoDB connector with change streams, aggregation pipeline support, and Atlas search integration', category: 'database', tier: 'verified', author: 'Sysilo', version: '2.1.0', installCount: 2100, avgRating: 4.6, ratingCount: 98, pricing: 'free', tags: ['nosql', 'document'] },
  { id: '7', name: 'AWS IoT Core', description: 'IoT device data ingestion with MQTT/HTTPS, device shadow sync, and rule engine integration', category: 'iot', tier: 'verified', author: 'CloudBridge Labs', version: '1.5.2', installCount: 980, avgRating: 4.4, ratingCount: 45, pricing: 'usage_based', monthlyPrice: 0.001, tags: ['iot', 'aws', 'mqtt'] },
  { id: '8', name: 'Bloomberg Data License', description: 'Real-time and historical market data feed connector for financial institutions', category: 'api', tier: 'premium', author: 'FinDataTech', version: '1.0.3', installCount: 320, avgRating: 4.9, ratingCount: 28, pricing: 'flat_rate', monthlyPrice: 499, tags: ['finance', 'market-data'] },
];

const TIER_COLORS: Record<ListingTier, string> = {
  free: 'bg-gray-700 text-gray-300',
  verified: 'bg-blue-900/50 text-blue-300 border border-blue-700',
  premium: 'bg-amber-900/50 text-amber-300 border border-amber-700',
};

const CATEGORY_ICONS: Record<string, string> = {
  database: 'cylinder',
  saas: 'cloud',
  api: 'globe',
  messaging: 'zap',
  file: 'file',
  iot: 'cpu',
  erp: 'building',
};

export function MarketplacePage() {
  const [search, setSearch] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedTier, setSelectedTier] = useState<string>('all');
  const [sortBy, setSortBy] = useState<'installs' | 'rating' | 'newest'>('installs');

  const categories = ['all', 'database', 'saas', 'api', 'messaging', 'file', 'iot', 'erp'];

  const filtered = MOCK_LISTINGS
    .filter(l => selectedCategory === 'all' || l.category === selectedCategory)
    .filter(l => selectedTier === 'all' || l.tier === selectedTier)
    .filter(l => !search || l.name.toLowerCase().includes(search.toLowerCase()) || l.tags.some(t => t.includes(search.toLowerCase())))
    .sort((a, b) => {
      if (sortBy === 'installs') return b.installCount - a.installCount;
      if (sortBy === 'rating') return b.avgRating - a.avgRating;
      return 0;
    });

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Connector Marketplace</h1>
          <p className="text-sm text-gray-400 mt-1">{MOCK_LISTINGS.length} connectors available &middot; {MOCK_LISTINGS.reduce((s, l) => s + l.installCount, 0).toLocaleString()} total installs</p>
        </div>
        <button className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium transition-colors">
          Publish Connector
        </button>
      </div>

      {/* Search & Filters */}
      <div className="flex flex-wrap gap-3 items-center">
        <input
          type="text"
          placeholder="Search connectors..."
          value={search}
          onChange={e => setSearch(e.target.value)}
          className="flex-1 min-w-[280px] px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-white placeholder-gray-500 text-sm focus:outline-none focus:border-blue-500"
        />
        <select value={selectedCategory} onChange={e => setSelectedCategory(e.target.value)} className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-300 text-sm">
          {categories.map(c => <option key={c} value={c}>{c === 'all' ? 'All Categories' : c.charAt(0).toUpperCase() + c.slice(1)}</option>)}
        </select>
        <select value={selectedTier} onChange={e => setSelectedTier(e.target.value)} className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-300 text-sm">
          <option value="all">All Tiers</option>
          <option value="free">Free</option>
          <option value="verified">Verified</option>
          <option value="premium">Premium</option>
        </select>
        <select value={sortBy} onChange={e => setSortBy(e.target.value as typeof sortBy)} className="px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-300 text-sm">
          <option value="installs">Most Installed</option>
          <option value="rating">Highest Rated</option>
          <option value="newest">Newest</option>
        </select>
      </div>

      {/* Listings Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {filtered.map(listing => (
          <div key={listing.id} className="bg-gray-800/50 border border-gray-700 rounded-xl p-5 hover:border-gray-600 transition-colors group cursor-pointer">
            <div className="flex items-start justify-between mb-3">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-gray-700 rounded-lg flex items-center justify-center text-gray-400 text-xs font-mono">
                  {listing.category.slice(0, 3).toUpperCase()}
                </div>
                <div>
                  <h3 className="text-white font-semibold group-hover:text-blue-400 transition-colors">{listing.name}</h3>
                  <p className="text-xs text-gray-500">by {listing.author} &middot; v{listing.version}</p>
                </div>
              </div>
              <span className={`text-xs px-2 py-0.5 rounded-full ${TIER_COLORS[listing.tier]}`}>
                {listing.tier}
              </span>
            </div>
            <p className="text-sm text-gray-400 mb-4 line-clamp-2">{listing.description}</p>
            <div className="flex items-center justify-between text-xs text-gray-500">
              <div className="flex items-center gap-3">
                <span>{listing.installCount.toLocaleString()} installs</span>
                <span className="text-yellow-400">{'*'.repeat(Math.round(listing.avgRating))} {listing.avgRating}</span>
                <span>({listing.ratingCount})</span>
              </div>
              <span className={listing.pricing === 'free' ? 'text-green-400' : 'text-gray-400'}>
                {listing.pricing === 'free' ? 'Free' : listing.pricing === 'flat_rate' ? `$${listing.monthlyPrice}/mo` : 'Usage-based'}
              </span>
            </div>
            <div className="flex flex-wrap gap-1.5 mt-3">
              {listing.tags.map(tag => (
                <span key={tag} className="text-xs px-2 py-0.5 bg-gray-700/50 text-gray-400 rounded">{tag}</span>
              ))}
            </div>
          </div>
        ))}
      </div>

      {filtered.length === 0 && (
        <div className="text-center py-12 text-gray-500">
          <p className="text-lg">No connectors found</p>
          <p className="text-sm mt-1">Try adjusting your search or filters</p>
        </div>
      )}
    </div>
  );
}
