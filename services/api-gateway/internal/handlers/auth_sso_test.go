package handlers

import (
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

func TestSSOEmailDomainPolicy(t *testing.T) {
	if !emailAllowedForDomain("user@example.com", "example.com") {
		t.Fatal("expected domain policy allow")
	}
	if emailAllowedForDomain("user@other.com", "example.com") {
		t.Fatal("expected domain policy deny")
	}
}

func TestSSOTokenIncludesSessionVersionClaim(t *testing.T) {
	token, err := issueAccessToken(
		"secret",
		"sysilo",
		"user-1",
		[]string{"admin"},
		"active",
		3,
		"tenant-1",
		10*time.Minute,
	)
	if err != nil {
		t.Fatalf("issue token: %v", err)
	}

	parsed, err := jwt.Parse(token, func(token *jwt.Token) (interface{}, error) {
		return []byte("secret"), nil
	})
	if err != nil {
		t.Fatalf("parse token: %v", err)
	}

	claims, ok := parsed.Claims.(jwt.MapClaims)
	if !ok {
		t.Fatal("expected map claims")
	}
	if got := int(claims["session_version"].(float64)); got != 3 {
		t.Fatalf("expected session version 3, got %d", got)
	}
	if got := claims["token_type"].(string); got != "access" {
		t.Fatalf("expected access token type, got %q", got)
	}
}

func TestJITStateExpiryValidation(t *testing.T) {
	expired, err := issueAccessToken("secret", "sysilo", "user-1", []string{"viewer"}, "active", 1, "tenant-1", -1*time.Minute)
	if err != nil {
		t.Fatalf("issue token: %v", err)
	}
	_, err = jwt.Parse(expired, func(token *jwt.Token) (interface{}, error) {
		return []byte("secret"), nil
	})
	if err == nil {
		t.Fatal("expected expired token parse to fail")
	}
}
