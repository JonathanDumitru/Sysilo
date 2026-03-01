package handlers

import (
	"errors"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/sysilo/sysilo/services/api-gateway/internal/auth"
	"go.uber.org/zap"
)

func (h *Handler) StartSSO(w http.ResponseWriter, r *http.Request) {
	tenantID := r.URL.Query().Get("tenant_id")
	domain := r.URL.Query().Get("domain")
	if tenantID == "" || domain == "" {
		respondError(w, http.StatusBadRequest, "tenant_id and domain are required")
		return
	}

	oidcClient, err := h.newOIDCClientFromEnv()
	if err != nil {
		h.Logger.Error("SSO is not configured", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "sso is not configured")
		return
	}

	state, err := auth.SignSSOState(auth.SSOState{
		TenantID: tenantID,
		Domain:   strings.ToLower(domain),
		Expires:  time.Now().Add(5 * time.Minute).Unix(),
	}, h.ssoStateSecret())
	if err != nil {
		h.Logger.Error("Failed to sign SSO state", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to start sso")
		return
	}

	redirect, err := oidcClient.StartAuthURL(r.Context(), state)
	if err != nil {
		h.Logger.Error("Failed to create OIDC redirect URL", zap.Error(err))
		respondError(w, http.StatusBadGateway, "failed to reach identity provider")
		return
	}
	respondJSON(w, http.StatusOK, map[string]string{"redirect_url": redirect, "state": state})
}

func (h *Handler) HandleSSOCallback(w http.ResponseWriter, r *http.Request) {
	code := r.URL.Query().Get("code")
	stateRaw := r.URL.Query().Get("state")
	if code == "" || stateRaw == "" {
		respondError(w, http.StatusBadRequest, "code and state are required")
		return
	}

	state, err := auth.VerifySSOState(stateRaw, h.ssoStateSecret(), time.Now())
	if err != nil {
		respondError(w, http.StatusUnauthorized, "invalid sso state")
		return
	}

	oidcClient, err := h.newOIDCClientFromEnv()
	if err != nil {
		h.Logger.Error("SSO is not configured", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "sso is not configured")
		return
	}

	oidcUser, err := oidcClient.ExchangeCode(r.Context(), code)
	if err != nil {
		h.Logger.Error("Failed to exchange OIDC code", zap.Error(err))
		respondError(w, http.StatusUnauthorized, "failed to validate identity")
		return
	}

	if !emailAllowedForDomain(oidcUser.Email, state.Domain) {
		respondError(w, http.StatusForbidden, "user domain is not allowed")
		return
	}

	user, err := h.DB.Users.UpsertJITBySubject(r.Context(), state.TenantID, oidcUser.Subject, oidcUser.Email, oidcUser.Name)
	if err != nil {
		h.Logger.Error("Failed to upsert JIT user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to provision user")
		return
	}

	if err := h.DB.Users.UpdateLastLogin(r.Context(), state.TenantID, user.ID); err != nil {
		h.Logger.Warn("Failed to update last login", zap.Error(err))
	}

	tokenManager, err := auth.NewTokenManager(os.Getenv("SYSILO_JWT_SECRET"), os.Getenv("SYSILO_JWT_ISSUER"))
	if err != nil {
		h.Logger.Error("Failed to initialize token manager", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to issue session")
		return
	}
	token, err := tokenManager.IssueAccessToken(auth.AccessTokenInput{
		UserID:         user.ID,
		TenantID:       state.TenantID,
		Roles:          user.Roles,
		Status:         user.Status,
		SessionVersion: user.SessionVersion,
		TTL:            15 * time.Minute,
	})
	if err != nil {
		h.Logger.Error("Failed to issue SSO access token", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to issue session")
		return
	}
	refreshToken, refreshHash, refreshExpiresAt, err := tokenManager.IssueRefreshToken(auth.RefreshTokenInput{
		UserID:         user.ID,
		TenantID:       state.TenantID,
		SessionVersion: user.SessionVersion,
		TTL:            7 * 24 * time.Hour,
	})
	if err != nil {
		h.Logger.Error("Failed to issue SSO refresh token", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to issue session")
		return
	}
	if err := h.DB.Users.StoreRefreshToken(r.Context(), state.TenantID, user.ID, refreshHash, refreshExpiresAt); err != nil {
		h.Logger.Error("Failed to persist refresh token", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to issue session")
		return
	}

	respondJSON(w, http.StatusOK, map[string]any{
		"access_token":  token,
		"refresh_token": refreshToken,
		"user":          user,
	})
}

func emailAllowedForDomain(email, domain string) bool {
	parts := strings.Split(strings.ToLower(email), "@")
	return len(parts) == 2 && parts[1] == strings.ToLower(domain)
}

func issueAccessToken(secret, issuer, userID string, roles []string, status string, sessionVersion int, tenantID string, ttl time.Duration) (string, error) {
	return auth.IssueAccessToken(secret, issuer, userID, roles, status, sessionVersion, tenantID, ttl)
}

func (h *Handler) ssoStateSecret() string {
	if v := os.Getenv("SYSILO_SSO_STATE_SECRET"); v != "" {
		return v
	}
	return "dev-sso-state-secret"
}

func (h *Handler) newOIDCClientFromEnv() (*auth.OIDCClient, error) {
	cfg := auth.OIDCConfig{
		IssuerURL:    os.Getenv("OIDC_ISSUER_URL"),
		ClientID:     os.Getenv("OIDC_CLIENT_ID"),
		ClientSecret: os.Getenv("OIDC_CLIENT_SECRET"),
		RedirectURL:  os.Getenv("OIDC_REDIRECT_URL"),
	}
	if cfg.IssuerURL == "" || cfg.ClientID == "" || cfg.ClientSecret == "" || cfg.RedirectURL == "" {
		return nil, errors.New("oidc env vars are incomplete")
	}
	return auth.NewOIDCClient(cfg), nil
}
