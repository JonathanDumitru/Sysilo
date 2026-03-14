import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  listPolicies,
  listViolations,
  resolveViolation,
  listApprovalRequests,
  getApprovalRequest,
  decideApprovalRequest,
  getAuditStats,
  getComplianceSummary,
  runComplianceAssessment,
  listStandards,
  type PolicyScope,
} from '../services/governance';

const POLICIES_KEY = ['governance', 'policies'] as const;
const VIOLATIONS_KEY = ['governance', 'violations'] as const;
const APPROVAL_REQUESTS_KEY = ['governance', 'approval-requests'] as const;
const AUDIT_STATS_KEY = ['governance', 'audit-stats'] as const;
const COMPLIANCE_SUMMARY_KEY = ['governance', 'compliance-summary'] as const;
const STANDARDS_KEY = ['governance', 'standards'] as const;

export function usePolicies(scope?: PolicyScope) {
  return useQuery({
    queryKey: [...POLICIES_KEY, scope],
    queryFn: () => listPolicies(scope),
  });
}

export function useViolations(status?: string, limit?: number) {
  return useQuery({
    queryKey: [...VIOLATIONS_KEY, status, limit],
    queryFn: () => listViolations(status, undefined, limit),
    refetchInterval: 30_000,
  });
}

export function useResolveViolation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, note }: { id: string; note: string }) => resolveViolation(id, note),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: VIOLATIONS_KEY });
    },
  });
}

export function useApprovalRequests(status?: string) {
  return useQuery({
    queryKey: [...APPROVAL_REQUESTS_KEY, status],
    queryFn: () => listApprovalRequests(status),
    refetchInterval: 15_000,
  });
}

export function useApprovalRequest(id: string | undefined) {
  return useQuery({
    queryKey: [...APPROVAL_REQUESTS_KEY, id],
    queryFn: () => getApprovalRequest(id!),
    enabled: !!id,
  });
}

export function useDecideApproval() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, decision, comment }: { id: string; decision: 'approved' | 'rejected'; comment?: string }) =>
      decideApprovalRequest(id, decision, comment),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: APPROVAL_REQUESTS_KEY });
    },
  });
}

export function useAuditStats() {
  return useQuery({
    queryKey: AUDIT_STATS_KEY,
    queryFn: getAuditStats,
    staleTime: 60_000,
  });
}

export function useComplianceSummary() {
  return useQuery({
    queryKey: COMPLIANCE_SUMMARY_KEY,
    queryFn: getComplianceSummary,
    staleTime: 60_000,
  });
}

export function useRunAssessment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (frameworkId: string) => runComplianceAssessment(frameworkId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: COMPLIANCE_SUMMARY_KEY });
    },
  });
}

export function useStandards(category?: string) {
  return useQuery({
    queryKey: [...STANDARDS_KEY, category],
    queryFn: () => listStandards(category),
  });
}
