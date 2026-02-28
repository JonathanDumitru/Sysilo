import { X, Zap, ArrowRight } from 'lucide-react';
import { useCheckout } from '../../hooks/useBilling';
import { usePlan } from '../../hooks/usePlan';

interface UpgradeModalProps {
  isOpen: boolean;
  onClose: () => void;
  feature: string;
  requiredPlan?: string;
}

const featureLabels: Record<string, string> = {
  governance_enabled: 'Governance',
  compliance_enabled: 'Compliance',
  rationalization_enabled: 'Rationalization',
  ai_enabled: 'AI Features',
  advanced_ops_enabled: 'Advanced Operations',
};

export function UpgradeModal({ isOpen, onClose, feature, requiredPlan = 'business' }: UpgradeModalProps) {
  const { planName } = usePlan();
  const checkout = useCheckout();

  if (!isOpen) return null;

  const featureLabel = featureLabels[feature] || feature;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />
      <div className="relative bg-white rounded-xl shadow-xl w-full max-w-md mx-4">
        <div className="flex items-center justify-between p-4 border-b border-gray-100">
          <div className="flex items-center gap-2">
            <Zap className="w-5 h-5 text-amber-500" />
            <h2 className="text-lg font-semibold text-gray-900">Upgrade Required</h2>
          </div>
          <button onClick={onClose} className="p-2 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-gray-100">
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <p className="text-gray-600">
            <span className="font-medium text-gray-900">{featureLabel}</span> is not available
            on your current <span className="font-medium">{planName}</span> plan.
          </p>

          <div className="bg-primary-50 border border-primary-100 rounded-lg p-4">
            <p className="text-sm text-primary-800">
              Upgrade to the <span className="font-semibold capitalize">{requiredPlan}</span> plan
              to unlock {featureLabel} and more.
            </p>
          </div>
        </div>

        <div className="flex items-center justify-end gap-3 p-4 border-t border-gray-100">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
          >
            Maybe later
          </button>
          <button
            onClick={() => checkout.mutate(requiredPlan)}
            disabled={checkout.isPending}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg disabled:opacity-50"
          >
            Upgrade now
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}
