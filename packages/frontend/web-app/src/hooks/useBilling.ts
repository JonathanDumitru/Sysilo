import { useMutation, useQuery } from '@tanstack/react-query';
import { listPlans, createCheckoutSession, createPortalSession, getSubscription } from '../services/billing';

export function useAvailablePlans() {
  return useQuery({
    queryKey: ['plans'],
    queryFn: listPlans,
    staleTime: 300_000,
  });
}

export function useCheckout() {
  return useMutation({
    mutationFn: (planName: string) => createCheckoutSession(planName),
    onSuccess: (data) => {
      if (data.checkout_url) {
        window.location.href = data.checkout_url;
      }
    },
  });
}

export function useBillingPortal() {
  return useMutation({
    mutationFn: () => createPortalSession(),
    onSuccess: (data) => {
      if (data.portal_url) {
        window.location.href = data.portal_url;
      }
    },
  });
}

export function useSubscription() {
  return useQuery({
    queryKey: ['subscription'],
    queryFn: getSubscription,
    staleTime: 60_000,
  });
}
