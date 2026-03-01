package auth

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

func TestOIDCStateSignAndVerify(t *testing.T) {
	now := time.Now()
	token, err := SignSSOState(SSOState{
		TenantID: "t-1",
		Domain:   "example.com",
		Expires:  now.Add(2 * time.Minute).Unix(),
	}, "secret")
	if err != nil {
		t.Fatalf("sign state: %v", err)
	}

	state, err := VerifySSOState(token, "secret", now)
	if err != nil {
		t.Fatalf("verify state: %v", err)
	}
	if state.TenantID != "t-1" || state.Domain != "example.com" {
		t.Fatalf("unexpected state: %+v", state)
	}
}

func TestOIDCStartAuthURLIncludesState(t *testing.T) {
	c := NewOIDCClient(OIDCConfig{
		IssuerURL:    "http://issuer.test",
		ClientID:     "client-id",
		ClientSecret: "secret",
		RedirectURL:  "http://localhost/callback",
	})
	c.httpClient = httpClientWithDiscovery(t, "http://issuer.test/auth", "http://issuer.test/token", nil)

	redirect, err := c.StartAuthURL(context.Background(), "sso-state")
	if err != nil {
		t.Fatalf("start auth url: %v", err)
	}
	u, err := url.Parse(redirect)
	if err != nil {
		t.Fatalf("parse redirect: %v", err)
	}
	if got := u.Query().Get("state"); got != "sso-state" {
		t.Fatalf("expected state in auth URL, got %q", got)
	}
}

func TestOIDCExchangeCodeParsesIDToken(t *testing.T) {
	issuer := httptest.NewServer(nil)
	defer issuer.Close()

	mux := http.NewServeMux()
	issuer.Config.Handler = mux

	cfg := OIDCConfig{
		IssuerURL:    issuer.URL,
		ClientID:     "client-id",
		ClientSecret: "super-secret",
		RedirectURL:  "http://localhost/callback",
	}
	c := NewOIDCClient(cfg)
	c.httpClient = issuer.Client()

	mux.HandleFunc("/.well-known/openid-configuration", func(w http.ResponseWriter, r *http.Request) {
		_ = json.NewEncoder(w).Encode(map[string]string{
			"authorization_endpoint": issuer.URL + "/auth",
			"token_endpoint":         issuer.URL + "/token",
		})
	})
	mux.HandleFunc("/token", func(w http.ResponseWriter, r *http.Request) {
		idToken := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
			"iss":   issuer.URL,
			"aud":   "client-id",
			"sub":   "idp-subject",
			"email": "jit@example.com",
			"name":  "JIT User",
			"exp":   time.Now().Add(1 * time.Hour).Unix(),
		})
		signed, err := idToken.SignedString([]byte("super-secret"))
		if err != nil {
			t.Fatalf("sign token: %v", err)
		}
		_ = json.NewEncoder(w).Encode(map[string]string{"id_token": signed})
	})

	user, err := c.ExchangeCode(context.Background(), "code-123")
	if err != nil {
		t.Fatalf("exchange code: %v", err)
	}
	if user.Subject != "idp-subject" || user.Email != "jit@example.com" {
		t.Fatalf("unexpected user: %+v", user)
	}
}

func httpClientWithDiscovery(t *testing.T, authURL, tokenURL string, base *http.Client) *http.Client {
	t.Helper()
	if base == nil {
		base = &http.Client{}
	}
	rt := base.Transport
	if rt == nil {
		rt = http.DefaultTransport
	}
	return &http.Client{
		Transport: roundTripperFunc(func(r *http.Request) (*http.Response, error) {
			if strings.HasSuffix(r.URL.Path, "/.well-known/openid-configuration") {
				rec := httptest.NewRecorder()
				_ = json.NewEncoder(rec).Encode(map[string]string{
					"authorization_endpoint": authURL,
					"token_endpoint":         tokenURL,
				})
				return rec.Result(), nil
			}
			return rt.RoundTrip(r)
		}),
	}
}

type roundTripperFunc func(*http.Request) (*http.Response, error)

func (f roundTripperFunc) RoundTrip(r *http.Request) (*http.Response, error) {
	return f(r)
}
