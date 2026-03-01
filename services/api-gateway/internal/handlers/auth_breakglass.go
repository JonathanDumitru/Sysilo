package handlers

import (
	"crypto/sha256"
	"crypto/subtle"
	"encoding/hex"
	"encoding/json"
	"errors"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/sysilo/sysilo/services/api-gateway/internal/auth"
	"github.com/sysilo/sysilo/services/api-gateway/internal/db"
	"go.uber.org/zap"
)

const breakglassChallengeTTL = 2 * time.Minute

func (h *Handler) StartBreakglassLogin(w http.ResponseWriter, r *http.Request) {
	var req struct {
		TenantID string `json:"tenant_id"`
		Email    string `json:"email"`
		Password string `json:"password"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	req.Email = strings.ToLower(strings.TrimSpace(req.Email))
	if req.TenantID == "" || req.Email == "" || req.Password == "" {
		respondError(w, http.StatusBadRequest, "tenant_id, email, and password are required")
		return
	}

	user, err := h.DB.Users.GetByEmail(r.Context(), req.TenantID, req.Email)
	if err != nil {
		h.Logger.Error("Failed to query breakglass user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to evaluate user")
		return
	}
	if !breakglassEligible(user) {
		respondError(w, http.StatusForbidden, "breakglass is not allowed for this account")
		return
	}
	if !verifyBreakglassPassword(user.PasswordHash.String, req.Password) {
		respondError(w, http.StatusUnauthorized, "invalid credentials")
		return
	}

	challenge, err := issueBreakglassChallenge(os.Getenv("SYSILO_JWT_SECRET"), os.Getenv("SYSILO_JWT_ISSUER"), user.ID, req.TenantID)
	if err != nil {
		h.Logger.Error("Failed to issue breakglass challenge", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to start breakglass login")
		return
	}

	respondJSON(w, http.StatusOK, map[string]any{
		"challenge_token": challenge,
		"expires_in":      int(breakglassChallengeTTL.Seconds()),
	})
}

func (h *Handler) CompleteBreakglassLogin(w http.ResponseWriter, r *http.Request) {
	var req struct {
		ChallengeToken string `json:"challenge_token"`
		Reason         string `json:"reason"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid request body")
		return
	}
	if strings.TrimSpace(req.Reason) == "" {
		respondError(w, http.StatusBadRequest, "reason is required for breakglass usage")
		return
	}

	tenantID, userID, err := verifyBreakglassChallenge(os.Getenv("SYSILO_JWT_SECRET"), os.Getenv("SYSILO_JWT_ISSUER"), req.ChallengeToken)
	if err != nil {
		respondError(w, http.StatusUnauthorized, "invalid breakglass challenge")
		return
	}

	user, err := h.DB.Users.GetByID(r.Context(), tenantID, userID)
	if err != nil {
		h.Logger.Error("Failed to load breakglass user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to load user")
		return
	}
	if !breakglassEligible(user) {
		respondError(w, http.StatusForbidden, "breakglass is not allowed for this account")
		return
	}

	tokenManager, err := auth.NewTokenManager(os.Getenv("SYSILO_JWT_SECRET"), os.Getenv("SYSILO_JWT_ISSUER"))
	if err != nil {
		h.Logger.Error("Missing JWT config", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "authentication is not configured")
		return
	}

	refreshToken, refreshHash, refreshExp, err := tokenManager.IssueRefreshToken(auth.RefreshTokenInput{
		UserID:         user.ID,
		TenantID:       tenantID,
		SessionVersion: user.SessionVersion,
		TTL:            7 * 24 * time.Hour,
	})
	if err != nil {
		h.Logger.Error("Failed to issue breakglass refresh token", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to issue session")
		return
	}
	if err := h.DB.Users.StoreRefreshToken(r.Context(), tenantID, user.ID, refreshHash, refreshExp); err != nil {
		h.Logger.Error("Failed to persist breakglass refresh token", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to issue session")
		return
	}
	accessToken, err := tokenManager.IssueAccessToken(auth.AccessTokenInput{
		UserID:         user.ID,
		TenantID:       tenantID,
		Roles:          user.Roles,
		Status:         user.Status,
		SessionVersion: user.SessionVersion,
		TTL:            15 * time.Minute,
	})
	if err != nil {
		h.Logger.Error("Failed to issue breakglass access token", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to issue session")
		return
	}

	if err := h.DB.Users.RecordBreakglassLogin(r.Context(), tenantID, user.ID, strings.TrimSpace(req.Reason)); err != nil {
		h.Logger.Error("Failed to record breakglass audit event", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to finalize breakglass login")
		return
	}

	respondJSON(w, http.StatusOK, map[string]any{
		"access_token":  accessToken,
		"refresh_token": refreshToken,
		"user":          user,
	})
}

func breakglassEligible(user *db.User) bool {
	if user == nil {
		return false
	}
	if user.Status != "active" {
		return false
	}
	if user.AuthSource != "local" {
		return false
	}
	if !user.BreakglassEligible {
		return false
	}
	if !hasRole(user.Roles, "admin") {
		return false
	}
	if !user.PasswordHash.Valid || user.PasswordHash.String == "" {
		return false
	}
	return true
}

func hasRole(roles []string, role string) bool {
	for _, candidate := range roles {
		if candidate == role {
			return true
		}
	}
	return false
}

func verifyBreakglassPassword(storedHash string, password string) bool {
	parts := strings.Split(storedHash, "$")
	if len(parts) != 3 || parts[0] != "sha256" {
		return false
	}
	salt := parts[1]
	expectedHex := parts[2]
	calculated := sha256.Sum256([]byte(salt + ":" + password))
	calculatedHex := hex.EncodeToString(calculated[:])
	return subtle.ConstantTimeCompare([]byte(calculatedHex), []byte(expectedHex)) == 1
}

func issueBreakglassChallenge(secret, issuer, userID, tenantID string) (string, error) {
	if secret == "" {
		return "", errors.New("missing SYSILO_JWT_SECRET")
	}
	if issuer == "" {
		issuer = "sysilo"
	}
	now := time.Now()
	claims := jwt.MapClaims{
		"iss":        issuer,
		"sub":        userID,
		"tenant_id":  tenantID,
		"token_type": "breakglass_challenge",
		"exp":        now.Add(breakglassChallengeTTL).Unix(),
		"iat":        now.Unix(),
	}
	return jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString([]byte(secret))
}

func verifyBreakglassChallenge(secret, issuer, token string) (tenantID, userID string, err error) {
	if secret == "" {
		return "", "", errors.New("missing SYSILO_JWT_SECRET")
	}
	if issuer == "" {
		issuer = "sysilo"
	}
	parsed, err := jwt.Parse(token, func(token *jwt.Token) (interface{}, error) {
		if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, jwt.ErrSignatureInvalid
		}
		return []byte(secret), nil
	})
	if err != nil {
		return "", "", err
	}
	claims, ok := parsed.Claims.(jwt.MapClaims)
	if !ok || !parsed.Valid {
		return "", "", errors.New("invalid breakglass challenge claims")
	}
	if claimIssuer, _ := claims["iss"].(string); claimIssuer != issuer {
		return "", "", errors.New("challenge issuer mismatch")
	}
	if tokenType, _ := claims["token_type"].(string); tokenType != "breakglass_challenge" {
		return "", "", errors.New("invalid challenge token type")
	}
	userID, _ = claims["sub"].(string)
	tenantID, _ = claims["tenant_id"].(string)
	if userID == "" || tenantID == "" {
		return "", "", errors.New("challenge missing subject claims")
	}
	return tenantID, userID, nil
}
