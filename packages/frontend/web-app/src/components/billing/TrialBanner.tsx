import { Clock, ArrowRight } from 'lucide-react';
import { usePlan } from '../../hooks/usePlan';
import { useNavigate } from 'react-router-dom';

export function TrialBanner() {
  const { isTrial, trialDaysLeft, isSuspended } = usePlan();
  const navigate = useNavigate();

  if (isSuspended) {
    return (
      <div className="bg-red-600 text-white px-4 py-2 text-sm flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Clock className="w-4 h-4" />
          <span>Your account is suspended. Upgrade to restore full access.</span>
        </div>
        <button
          onClick={() => navigate('/settings?tab=billing')}
          className="flex items-center gap-1 text-white/90 hover:text-white font-medium"
        >
          Upgrade <ArrowRight className="w-3 h-3" />
        </button>
      </div>
    );
  }

  if (!isTrial || trialDaysLeft === null) return null;

  const urgentClass = trialDaysLeft <= 3 ? 'bg-amber-500' : 'bg-primary-600';

  return (
    <div className={`${urgentClass} text-white px-4 py-2 text-sm flex items-center justify-between`}>
      <div className="flex items-center gap-2">
        <Clock className="w-4 h-4" />
        <span>
          {trialDaysLeft === 0
            ? 'Your trial expires today!'
            : `${trialDaysLeft} day${trialDaysLeft === 1 ? '' : 's'} left in your trial`}
        </span>
      </div>
      <button
        onClick={() => navigate('/settings?tab=billing')}
        className="flex items-center gap-1 text-white/90 hover:text-white font-medium"
      >
        Choose a plan <ArrowRight className="w-3 h-3" />
      </button>
    </div>
  );
}
