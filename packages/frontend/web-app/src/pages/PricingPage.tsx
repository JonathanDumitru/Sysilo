import { Check, X as XIcon, Loader2 } from 'lucide-react';
import { useAvailablePlans, useCheckout } from '../hooks/useBilling';
import { usePlan } from '../hooks/usePlan';
import type { Plan } from '../services/billing';

const tierHighlights: Record<string, string[]> = {
  team: ['5 users', '10 integrations', '500 runs/mo', 'Community support'],
  business: ['15 users', '50 integrations', '5,000 runs/mo', 'Governance & AI', 'Email support (48h)'],
  enterprise: ['Unlimited users', 'Unlimited integrations', 'Unlimited runs', 'Full platform', 'Dedicated support (4h)'],
};

function PlanCard({ plan, isCurrent }: { plan: Plan; isCurrent: boolean }) {
  const checkout = useCheckout();
  const isBusiness = plan.name === 'business';
  const isEnterprise = plan.name === 'enterprise';

  return (
    <div className={`bg-surface-raised/80 backdrop-blur-glass rounded-xl shadow-glass border-2 p-6 flex flex-col ${isBusiness ? 'border-primary-500 ring-2 ring-primary-900/50' : 'border-surface-border'}`}>
      {isBusiness && (
        <div className="text-xs font-semibold text-primary-400 uppercase tracking-wide mb-2">Most Popular</div>
      )}
      <h3 className="text-xl font-bold text-white">{plan.display_name}</h3>
      <div className="mt-2">
        {isEnterprise ? (
          <span className="text-3xl font-bold text-white">Custom</span>
        ) : (
          <>
            <span className="text-3xl font-bold text-white">${(plan.price_cents / 100).toLocaleString()}</span>
            <span className="text-gray-500">/mo</span>
          </>
        )}
      </div>
      <p className="mt-2 text-sm text-gray-500">{plan.description}</p>

      <ul className="mt-6 space-y-3 flex-1">
        {(tierHighlights[plan.name] || []).map((item) => (
          <li key={item} className="flex items-center gap-2 text-sm text-gray-300">
            <Check className="w-4 h-4 text-green-500 flex-shrink-0" />
            {item}
          </li>
        ))}
      </ul>

      <div className="mt-6">
        {isCurrent ? (
          <button disabled className="w-full px-4 py-2 text-sm font-medium text-gray-500 bg-gray-700/50 rounded-lg">
            Current Plan
          </button>
        ) : isEnterprise ? (
          <a href="mailto:sales@sysilo.io" className="block w-full px-4 py-2 text-sm font-medium text-center text-primary-400 border border-primary-600 rounded-lg hover:bg-white/5">
            Contact Sales
          </a>
        ) : (
          <button
            onClick={() => checkout.mutate(plan.name)}
            disabled={checkout.isPending}
            className="w-full flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {checkout.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
            Upgrade to {plan.display_name}
          </button>
        )}
      </div>
    </div>
  );
}

export function PricingPage() {
  const { data, isLoading } = useAvailablePlans();
  const { plan: currentPlan } = usePlan();

  return (
    <div className="space-y-6">
      <div className="text-center">
        <h1 className="text-2xl font-bold text-white">Choose Your Plan</h1>
        <p className="text-gray-500 mt-1">Scale your integration platform as you grow</p>
      </div>

      {isLoading ? (
        <div className="flex justify-center py-12">
          <Loader2 className="w-8 h-8 animate-spin text-gray-400" />
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl mx-auto">
          {data?.plans.map((plan) => (
            <PlanCard key={plan.id} plan={plan} isCurrent={currentPlan?.name === plan.name} />
          ))}
        </div>
      )}
    </div>
  );
}
