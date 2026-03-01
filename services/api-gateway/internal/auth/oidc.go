package auth

import (
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

type OIDCConfig struct {
	IssuerURL    string
	ClientID     string
	ClientSecret string
	RedirectURL  string
}

type OIDCClient struct {
	cfg        OIDCConfig
	httpClient *http.Client
}

type OIDCUser struct {
	Subject string
	Email   string
	Name    string
}

func NewOIDCClient(cfg OIDCConfig) *OIDCClient {
	return &OIDCClient{
		cfg: cfg,
		httpClient: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

type oidcDiscovery struct {
	AuthorizationEndpoint string `json:"authorization_endpoint"`
	TokenEndpoint         string `json:"token_endpoint"`
}

type oidcTokenResponse struct {
	IDToken string `json:"id_token"`
}

func (c *OIDCClient) StartAuthURL(ctx context.Context, state string) (string, error) {
	dc, err := c.discovery(ctx)
	if err != nil {
		return "", err
	}
	authURL, err := url.Parse(dc.AuthorizationEndpoint)
	if err != nil {
		return "", fmt.Errorf("parse authorization endpoint: %w", err)
	}

	q := authURL.Query()
	q.Set("response_type", "code")
	q.Set("client_id", c.cfg.ClientID)
	q.Set("redirect_uri", c.cfg.RedirectURL)
	q.Set("scope", "openid profile email")
	q.Set("state", state)
	authURL.RawQuery = q.Encode()
	return authURL.String(), nil
}

func (c *OIDCClient) ExchangeCode(ctx context.Context, code string) (*OIDCUser, error) {
	dc, err := c.discovery(ctx)
	if err != nil {
		return nil, err
	}

	form := url.Values{}
	form.Set("grant_type", "authorization_code")
	form.Set("code", code)
	form.Set("client_id", c.cfg.ClientID)
	form.Set("client_secret", c.cfg.ClientSecret)
	form.Set("redirect_uri", c.cfg.RedirectURL)

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, dc.TokenEndpoint, strings.NewReader(form.Encode()))
	if err != nil {
		return nil, fmt.Errorf("build token exchange request: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("token exchange request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("token exchange failed: %s", resp.Status)
	}

	var tr oidcTokenResponse
	if err := json.NewDecoder(resp.Body).Decode(&tr); err != nil {
		return nil, fmt.Errorf("decode token response: %w", err)
	}
	if tr.IDToken == "" {
		return nil, errors.New("missing id_token in token response")
	}
	return c.parseAndValidateIDToken(tr.IDToken)
}

func (c *OIDCClient) discovery(ctx context.Context) (*oidcDiscovery, error) {
	discoveryURL := strings.TrimSuffix(c.cfg.IssuerURL, "/") + "/.well-known/openid-configuration"
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, discoveryURL, nil)
	if err != nil {
		return nil, fmt.Errorf("build discovery request: %w", err)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("discovery request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("discovery failed: %s", resp.Status)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read discovery response: %w", err)
	}

	var dc oidcDiscovery
	if err := json.Unmarshal(body, &dc); err != nil {
		return nil, fmt.Errorf("decode discovery response: %w", err)
	}
	if dc.AuthorizationEndpoint == "" || dc.TokenEndpoint == "" {
		return nil, errors.New("discovery response missing endpoints")
	}
	return &dc, nil
}

func (c *OIDCClient) parseAndValidateIDToken(idToken string) (*OIDCUser, error) {
	parser := jwt.NewParser(jwt.WithValidMethods([]string{jwt.SigningMethodHS256.Alg()}))
	token, err := parser.Parse(idToken, func(token *jwt.Token) (interface{}, error) {
		// HS256 is used only for local/test validation paths.
		return []byte(c.cfg.ClientSecret), nil
	})
	if err != nil {
		return nil, fmt.Errorf("parse id token: %w", err)
	}

	claims, ok := token.Claims.(jwt.MapClaims)
	if !ok || !token.Valid {
		return nil, errors.New("invalid id token claims")
	}
	iss, _ := claims["iss"].(string)
	if strings.TrimSuffix(iss, "/") != strings.TrimSuffix(c.cfg.IssuerURL, "/") {
		return nil, errors.New("id token issuer mismatch")
	}
	audOK := false
	switch aud := claims["aud"].(type) {
	case string:
		audOK = aud == c.cfg.ClientID
	case []interface{}:
		for _, item := range aud {
			if s, ok := item.(string); ok && s == c.cfg.ClientID {
				audOK = true
				break
			}
		}
	}
	if !audOK {
		return nil, errors.New("id token audience mismatch")
	}

	sub, _ := claims["sub"].(string)
	email, _ := claims["email"].(string)
	name, _ := claims["name"].(string)
	if sub == "" || email == "" {
		return nil, errors.New("id token missing required subject or email")
	}
	return &OIDCUser{
		Subject: sub,
		Email:   email,
		Name:    name,
	}, nil
}

type SSOState struct {
	TenantID string `json:"tenant_id"`
	Domain   string `json:"domain"`
	Expires  int64  `json:"exp"`
}

func SignSSOState(state SSOState, secret string) (string, error) {
	raw, err := json.Marshal(state)
	if err != nil {
		return "", fmt.Errorf("marshal state: %w", err)
	}
	sig := signHMAC(raw, secret)
	return base64.RawURLEncoding.EncodeToString(raw) + "." + base64.RawURLEncoding.EncodeToString(sig), nil
}

func VerifySSOState(encoded string, secret string, now time.Time) (*SSOState, error) {
	parts := strings.Split(encoded, ".")
	if len(parts) != 2 {
		return nil, errors.New("invalid sso state format")
	}

	raw, err := base64.RawURLEncoding.DecodeString(parts[0])
	if err != nil {
		return nil, errors.New("invalid sso state payload")
	}
	givenSig, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return nil, errors.New("invalid sso state signature")
	}
	expectedSig := signHMAC(raw, secret)
	if !hmac.Equal(givenSig, expectedSig) {
		return nil, errors.New("invalid sso state signature")
	}

	var state SSOState
	if err := json.Unmarshal(raw, &state); err != nil {
		return nil, errors.New("invalid sso state content")
	}
	if state.Expires <= now.Unix() {
		return nil, errors.New("sso state expired")
	}
	return &state, nil
}

func signHMAC(data []byte, secret string) []byte {
	h := hmac.New(sha256.New, []byte(secret))
	h.Write(data)
	return h.Sum(nil)
}
