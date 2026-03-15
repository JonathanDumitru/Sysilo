/**
 * Sysilo Connector Marketplace SDK
 *
 * Tools for publishing, managing, and monetizing connectors
 * in the Sysilo Connector Marketplace.
 */

import { ConnectorMetadata, ConnectorCategory } from '../types';

// =============================================================================
// Marketplace Types
// =============================================================================

export type ListingTier = 'free' | 'verified' | 'premium';
export type ListingStatus = 'draft' | 'pending_review' | 'published' | 'suspended';
export type PricingModel = 'free' | 'usage_based' | 'flat_rate';
export type TemplateVertical =
  | 'healthcare'
  | 'finance'
  | 'manufacturing'
  | 'retail_ecommerce'
  | 'government'
  | 'education'
  | 'telecommunications'
  | 'energy'
  | 'logistics'
  | 'custom';

export interface MarketplaceListing {
  id: string;
  connectorId: string;
  publisherId: string;
  name: string;
  description: string;
  version: string;
  iconUrl?: string;
  category: ConnectorCategory;
  tags: string[];
  tier: ListingTier;
  pricing: PricingModel;
  pricePerCall?: number;
  monthlyPrice?: number;
  revenueSharePct: number;
  installCount: number;
  avgRating: number;
  ratingCount: number;
  status: ListingStatus;
  slaGuaranteed: boolean;
  documentationUrl?: string;
  sourceRepoUrl?: string;
  createdAt: string;
  updatedAt: string;
  publishedAt?: string;
}

export interface MarketplaceReview {
  id: string;
  listingId: string;
  reviewerId: string;
  rating: number;
  title: string;
  body: string;
  helpfulCount: number;
  createdAt: string;
}

export interface ConnectorAnalyticsSummary {
  listingId: string;
  installs: number;
  activeUsers: number;
  apiCalls: number;
  errorRate: number;
  revenueEarned: number;
  p50LatencyMs: number;
  p99LatencyMs: number;
}

export interface PublishRequest {
  connectorMetadata: ConnectorMetadata;
  description: string;
  iconUrl?: string;
  tags: string[];
  tier: ListingTier;
  pricing: PricingModel;
  pricePerCall?: number;
  monthlyPrice?: number;
  documentationUrl?: string;
  sourceRepoUrl?: string;
  slaGuaranteed?: boolean;
}

export interface IndustryTemplate {
  id: string;
  name: string;
  vertical: TemplateVertical;
  description: string;
  requiredConnectors: string[];
  complianceFrameworks: string[];
  estimatedSetupMinutes: number;
  installCount: number;
  avgRating: number;
}

// =============================================================================
// Marketplace Client
// =============================================================================

export class MarketplaceClient {
  private baseUrl: string;
  private apiKey: string;
  private publisherId: string;

  constructor(config: {
    baseUrl: string;
    apiKey: string;
    publisherId: string;
  }) {
    this.baseUrl = config.baseUrl.replace(/\/$/, '');
    this.apiKey = config.apiKey;
    this.publisherId = config.publisherId;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${this.apiKey}`,
      'X-Publisher-ID': this.publisherId,
    };

    const response = await fetch(url, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new MarketplaceError(
        `Marketplace API error (${response.status}): ${errorText}`,
        response.status
      );
    }

    return response.json() as Promise<T>;
  }

  // =========================================================================
  // Listing Management
  // =========================================================================

  async publishConnector(req: PublishRequest): Promise<MarketplaceListing> {
    return this.request<MarketplaceListing>('POST', '/marketplace/listings', {
      ...req,
      publisherId: this.publisherId,
    });
  }

  async getMyListings(): Promise<MarketplaceListing[]> {
    const result = await this.request<{ listings: MarketplaceListing[] }>(
      'GET',
      `/marketplace/listings?publisher_id=${this.publisherId}`
    );
    return result.listings;
  }

  async getListing(listingId: string): Promise<MarketplaceListing> {
    return this.request<MarketplaceListing>(
      'GET',
      `/marketplace/listings/${listingId}`
    );
  }

  async submitForReview(listingId: string): Promise<MarketplaceListing> {
    return this.request<MarketplaceListing>(
      'POST',
      `/marketplace/listings/${listingId}/submit`
    );
  }

  async updateListing(
    listingId: string,
    updates: Partial<PublishRequest>
  ): Promise<MarketplaceListing> {
    return this.request<MarketplaceListing>(
      'PUT',
      `/marketplace/listings/${listingId}`,
      updates
    );
  }

  // =========================================================================
  // Analytics
  // =========================================================================

  async getAnalytics(listingId: string): Promise<ConnectorAnalyticsSummary> {
    return this.request<ConnectorAnalyticsSummary>(
      'GET',
      `/marketplace/listings/${listingId}/analytics`
    );
  }

  async getRevenueSummary(
    period?: string
  ): Promise<{
    totalRevenue: number;
    revenueByListing: Record<string, number>;
    period: string;
  }> {
    const query = period ? `?period=${period}` : '';
    return this.request('GET', `/marketplace/revenue${query}`);
  }

  // =========================================================================
  // Discovery
  // =========================================================================

  async searchListings(params: {
    query?: string;
    category?: ConnectorCategory;
    tier?: ListingTier;
    sortBy?: 'installs' | 'rating' | 'newest';
    limit?: number;
    offset?: number;
  }): Promise<{ listings: MarketplaceListing[]; total: number }> {
    const searchParams = new URLSearchParams();
    if (params.query) searchParams.set('q', params.query);
    if (params.category) searchParams.set('category', params.category);
    if (params.tier) searchParams.set('tier', params.tier);
    if (params.sortBy) searchParams.set('sort', params.sortBy);
    if (params.limit) searchParams.set('limit', String(params.limit));
    if (params.offset) searchParams.set('offset', String(params.offset));

    return this.request(
      'GET',
      `/marketplace/listings?${searchParams.toString()}`
    );
  }

  async getFeaturedListings(): Promise<MarketplaceListing[]> {
    const result = await this.request<{ featured: MarketplaceListing[] }>(
      'GET',
      '/marketplace/featured'
    );
    return result.featured;
  }

  // =========================================================================
  // Reviews
  // =========================================================================

  async getReviews(listingId: string): Promise<MarketplaceReview[]> {
    const result = await this.request<{ reviews: MarketplaceReview[] }>(
      'GET',
      `/marketplace/listings/${listingId}/reviews`
    );
    return result.reviews;
  }

  async addReview(
    listingId: string,
    review: { rating: number; title: string; body: string }
  ): Promise<MarketplaceReview> {
    return this.request<MarketplaceReview>(
      'POST',
      `/marketplace/listings/${listingId}/reviews`,
      review
    );
  }

  // =========================================================================
  // Industry Templates
  // =========================================================================

  async listTemplates(params?: {
    vertical?: TemplateVertical;
    search?: string;
  }): Promise<IndustryTemplate[]> {
    const searchParams = new URLSearchParams();
    if (params?.vertical) searchParams.set('vertical', params.vertical);
    if (params?.search) searchParams.set('search', params.search);

    const result = await this.request<{ templates: IndustryTemplate[] }>(
      'GET',
      `/marketplace/templates?${searchParams.toString()}`
    );
    return result.templates;
  }

  async deployTemplate(
    templateId: string,
    customizations?: Record<string, unknown>
  ): Promise<{ deploymentId: string; status: string }> {
    return this.request(
      'POST',
      `/marketplace/templates/${templateId}/deploy`,
      { customizations }
    );
  }
}

// =============================================================================
// Errors
// =============================================================================

export class MarketplaceError extends Error {
  constructor(
    message: string,
    public readonly statusCode: number
  ) {
    super(message);
    this.name = 'MarketplaceError';
  }
}
