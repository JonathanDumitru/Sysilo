import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  getTimeSummary,
  listTimeAssessments,
  listApplications,
  listScenarios,
  createScenario,
  analyzeScenario,
  listRecommendations,
  generateRecommendations,
  getPortfolioAnalytics,
  bulkCalculateScores,
  type ListApplicationsParams,
  type CreateScenarioRequest,
  type TimeSummary,
  type TimeAssessment,
  type Application,
  type Scenario,
  type Recommendation,
  type PortfolioAnalytics,
} from '../services/rationalization';

// Query keys
export const rationalizationKeys = {
  all: ['rationalization'] as const,
  timeSummary: () => [...rationalizationKeys.all, 'time-summary'] as const,
  timeAssessments: () => [...rationalizationKeys.all, 'time-assessments'] as const,
  applications: (params?: ListApplicationsParams) =>
    [...rationalizationKeys.all, 'applications', params] as const,
  scenarios: () => [...rationalizationKeys.all, 'scenarios'] as const,
  recommendations: () => [...rationalizationKeys.all, 'recommendations'] as const,
  portfolioAnalytics: () => [...rationalizationKeys.all, 'portfolio-analytics'] as const,
};

// TIME summary (quadrant counts)
export function useTimeSummary() {
  return useQuery({
    queryKey: rationalizationKeys.timeSummary(),
    queryFn: getTimeSummary,
    staleTime: 30_000,
  });
}

// TIME assessments (individual application assessments)
export function useTimeAssessments() {
  return useQuery({
    queryKey: rationalizationKeys.timeAssessments(),
    queryFn: listTimeAssessments,
  });
}

// Applications list
export function useApplications(params?: ListApplicationsParams) {
  return useQuery({
    queryKey: rationalizationKeys.applications(params),
    queryFn: () => listApplications(params),
  });
}

// Scenarios list
export function useScenarios() {
  return useQuery({
    queryKey: rationalizationKeys.scenarios(),
    queryFn: listScenarios,
  });
}

// Create scenario
export function useCreateScenario() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CreateScenarioRequest) => createScenario(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: rationalizationKeys.scenarios() });
    },
  });
}

// Analyze scenario
export function useAnalyzeScenario() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => analyzeScenario(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: rationalizationKeys.scenarios() });
    },
  });
}

// Recommendations list
export function useRecommendations() {
  return useQuery({
    queryKey: rationalizationKeys.recommendations(),
    queryFn: listRecommendations,
    staleTime: 60_000,
  });
}

// Generate recommendations
export function useGenerateRecommendations() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: generateRecommendations,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: rationalizationKeys.recommendations() });
    },
  });
}

// Portfolio analytics
export function usePortfolioAnalytics() {
  return useQuery({
    queryKey: rationalizationKeys.portfolioAnalytics(),
    queryFn: getPortfolioAnalytics,
    staleTime: 30_000,
  });
}

// Bulk calculate scores
export function useBulkCalculateScores() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: bulkCalculateScores,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: rationalizationKeys.timeAssessments() });
      queryClient.invalidateQueries({ queryKey: rationalizationKeys.timeSummary() });
      queryClient.invalidateQueries({ queryKey: rationalizationKeys.portfolioAnalytics() });
    },
  });
}

// Re-export types for convenience
export type {
  TimeSummary,
  TimeAssessment,
  Application,
  Scenario,
  Recommendation,
  PortfolioAnalytics,
  ListApplicationsParams,
  CreateScenarioRequest,
};
