package handlers

import (
	"encoding/json"
	"net/http"
	"os"
	"time"

	"github.com/sysilo/sysilo/services/api-gateway/internal/auth"
	"go.uber.org/zap"
)

func (h *Handler) RefreshSession(w http.ResponseWriter, r *http.Request) {
	var req struct {
		RefreshToken string `json:"refresh_token"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if req.RefreshToken == "" {
		respondError(w, http.StatusBadRequest, "refresh_token is required")
		return
	}

	tokenManager, err := auth.NewTokenManager(os.Getenv("SYSILO_JWT_SECRET"), os.Getenv("SYSILO_JWT_ISSUER"))
	if err != nil {
		h.Logger.Error("Missing JWT config", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "authentication is not configured")
		return
	}

	claims, err := tokenManager.ParseRefreshToken(req.RefreshToken)
	if err != nil {
		respondError(w, http.StatusUnauthorized, "invalid refresh token")
		return
	}

	oldHash := auth.HashRefreshToken(req.RefreshToken)
	newRaw, newHash, newExpiresAt, err := tokenManager.IssueRefreshToken(auth.RefreshTokenInput{
		UserID:         claims.UserID,
		TenantID:       claims.TenantID,
		SessionVersion: claims.SessionVersion,
		TTL:            7 * 24 * time.Hour,
	})
	if err != nil {
		h.Logger.Error("Failed to issue rotated refresh token", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to refresh session")
		return
	}

	user, err := h.DB.Users.RotateRefreshToken(r.Context(), claims.TenantID, oldHash, newHash, newExpiresAt)
	if err != nil {
		h.Logger.Warn("Refresh token rotation failed", zap.Error(err))
		respondError(w, http.StatusUnauthorized, "invalid or expired refresh token")
		return
	}

	accessToken, err := tokenManager.IssueAccessToken(auth.AccessTokenInput{
		UserID:         user.ID,
		TenantID:       claims.TenantID,
		Roles:          user.Roles,
		Status:         user.Status,
		SessionVersion: user.SessionVersion,
		TTL:            15 * time.Minute,
	})
	if err != nil {
		h.Logger.Error("Failed to issue refreshed access token", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to refresh session")
		return
	}

	respondJSON(w, http.StatusOK, map[string]any{
		"access_token":  accessToken,
		"refresh_token": newRaw,
		"user":          user,
	})
}
