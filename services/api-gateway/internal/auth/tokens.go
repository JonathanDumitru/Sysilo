package auth

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"fmt"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

type TokenManager struct {
	secret []byte
	issuer string
}

func NewTokenManager(secret, issuer string) (*TokenManager, error) {
	if secret == "" {
		return nil, errors.New("missing SYSILO_JWT_SECRET")
	}
	if issuer == "" {
		issuer = "sysilo"
	}
	return &TokenManager{
		secret: []byte(secret),
		issuer: issuer,
	}, nil
}

type AccessTokenInput struct {
	UserID         string
	TenantID       string
	Roles          []string
	Status         string
	SessionVersion int
	TTL            time.Duration
}

func (m *TokenManager) IssueAccessToken(input AccessTokenInput) (string, error) {
	now := time.Now()
	claims := jwt.MapClaims{
		"iss":             m.issuer,
		"sub":             input.UserID,
		"tenant_id":       input.TenantID,
		"roles":           input.Roles,
		"status":          input.Status,
		"session_version": input.SessionVersion,
		"exp":             now.Add(input.TTL).Unix(),
		"iat":             now.Unix(),
		"token_type":      "access",
	}
	return jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString(m.secret)
}

type RefreshTokenInput struct {
	UserID         string
	TenantID       string
	SessionVersion int
	TTL            time.Duration
}

func (m *TokenManager) IssueRefreshToken(input RefreshTokenInput) (rawToken, tokenHash string, expiresAt time.Time, err error) {
	jti, err := randomString(24)
	if err != nil {
		return "", "", time.Time{}, fmt.Errorf("generate refresh token id: %w", err)
	}
	now := time.Now()
	expiresAt = now.Add(input.TTL)
	claims := jwt.MapClaims{
		"iss":             m.issuer,
		"sub":             input.UserID,
		"tenant_id":       input.TenantID,
		"session_version": input.SessionVersion,
		"exp":             expiresAt.Unix(),
		"iat":             now.Unix(),
		"jti":             jti,
		"token_type":      "refresh",
	}
	rawToken, err = jwt.NewWithClaims(jwt.SigningMethodHS256, claims).SignedString(m.secret)
	if err != nil {
		return "", "", time.Time{}, fmt.Errorf("sign refresh token: %w", err)
	}
	return rawToken, HashRefreshToken(rawToken), expiresAt, nil
}

type RefreshClaims struct {
	UserID         string
	TenantID       string
	SessionVersion int
	TokenID        string
}

func (m *TokenManager) ParseRefreshToken(token string) (*RefreshClaims, error) {
	claims, err := m.parseAndValidate(token, "refresh")
	if err != nil {
		return nil, err
	}
	sessionVersion, err := parseIntClaim(claims["session_version"])
	if err != nil {
		return nil, err
	}
	userID, _ := claims["sub"].(string)
	tenantID, _ := claims["tenant_id"].(string)
	tokenID, _ := claims["jti"].(string)
	if userID == "" || tenantID == "" || tokenID == "" {
		return nil, errors.New("refresh token missing claims")
	}
	return &RefreshClaims{
		UserID:         userID,
		TenantID:       tenantID,
		SessionVersion: sessionVersion,
		TokenID:        tokenID,
	}, nil
}

func (m *TokenManager) ParseAccessToken(token string) (jwt.MapClaims, error) {
	return m.parseAndValidate(token, "access")
}

func (m *TokenManager) parseAndValidate(tokenString string, expectedType string) (jwt.MapClaims, error) {
	parser := jwt.NewParser(jwt.WithValidMethods([]string{jwt.SigningMethodHS256.Alg()}))
	token, err := parser.Parse(tokenString, func(token *jwt.Token) (interface{}, error) {
		return m.secret, nil
	})
	if err != nil {
		return nil, fmt.Errorf("parse token: %w", err)
	}
	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok || !token.Valid {
		return nil, errors.New("invalid token claims")
	}
	if iss, _ := claims["iss"].(string); iss != m.issuer {
		return nil, errors.New("token issuer mismatch")
	}
	if tokenType, _ := claims["token_type"].(string); tokenType != expectedType {
		return nil, errors.New("unexpected token type")
	}
	return claims, nil
}

func HashRefreshToken(token string) string {
	sum := sha256.Sum256([]byte(token))
	return hex.EncodeToString(sum[:])
}

func IssueAccessToken(secret, issuer string, userID string, roles []string, status string, sessionVersion int, tenantID string, ttl time.Duration) (string, error) {
	manager, err := NewTokenManager(secret, issuer)
	if err != nil {
		return "", err
	}
	return manager.IssueAccessToken(AccessTokenInput{
		UserID:         userID,
		TenantID:       tenantID,
		Roles:          roles,
		Status:         status,
		SessionVersion: sessionVersion,
		TTL:            ttl,
	})
}

func parseIntClaim(v interface{}) (int, error) {
	switch n := v.(type) {
	case float64:
		return int(n), nil
	case int:
		return n, nil
	default:
		return 0, errors.New("invalid integer claim")
	}
}

func randomString(bytesLen int) (string, error) {
	b := make([]byte, bytesLen)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return base64.RawURLEncoding.EncodeToString(b), nil
}
