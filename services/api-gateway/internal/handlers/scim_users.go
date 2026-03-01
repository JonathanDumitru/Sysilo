package handlers

import (
	"encoding/json"
	"net/http"
	"os"
	"strings"

	"github.com/go-chi/chi/v5"
	"go.uber.org/zap"
)

type scimUserPayload struct {
	ExternalID string `json:"externalId"`
	UserName   string `json:"userName"`
	DisplayName string `json:"displayName"`
	Active     *bool  `json:"active"`
	Name       struct {
		GivenName  string `json:"givenName"`
		FamilyName string `json:"familyName"`
	} `json:"name"`
	Emails []struct {
		Value   string `json:"value"`
		Primary bool   `json:"primary"`
	} `json:"emails"`
}

func (h *Handler) SCIMCreateUser(w http.ResponseWriter, r *http.Request) {
	tenantID, ok := h.scimTenantAndAuth(w, r)
	if !ok {
		return
	}

	var req scimUserPayload
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid SCIM payload")
		return
	}

	email := scimEmail(req)
	subject := scimSubject(req)
	if subject == "" || email == "" {
		respondError(w, http.StatusBadRequest, "SCIM user must include externalId and email")
		return
	}

	active := true
	if req.Active != nil {
		active = *req.Active
	}

	user, err := h.DB.Users.UpsertSCIMBySubject(r.Context(), tenantID, subject, email, scimName(req), active)
	if err != nil {
		h.Logger.Error("Failed to upsert SCIM user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to provision SCIM user")
		return
	}

	respondJSON(w, http.StatusCreated, map[string]any{
		"id":       user.ID,
		"externalId": subject,
		"userName": user.Email,
		"active":   user.Status == "active",
	})
}

func (h *Handler) SCIMUpdateUser(w http.ResponseWriter, r *http.Request) {
	tenantID, ok := h.scimTenantAndAuth(w, r)
	if !ok {
		return
	}

	pathSubject := chi.URLParam(r, "userID")
	var req scimUserPayload
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		respondError(w, http.StatusBadRequest, "invalid SCIM payload")
		return
	}

	subject := scimSubject(req)
	if subject == "" {
		subject = pathSubject
	}
	email := scimEmail(req)
	if subject == "" || email == "" {
		respondError(w, http.StatusBadRequest, "SCIM update requires subject and email")
		return
	}

	active := true
	if req.Active != nil {
		active = *req.Active
	}

	user, err := h.DB.Users.UpsertSCIMBySubject(r.Context(), tenantID, subject, email, scimName(req), active)
	if err != nil {
		h.Logger.Error("Failed to update SCIM user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to update SCIM user")
		return
	}

	respondJSON(w, http.StatusOK, map[string]any{
		"id":         user.ID,
		"externalId": subject,
		"userName":   user.Email,
		"active":     user.Status == "active",
	})
}

func (h *Handler) Deactivate(w http.ResponseWriter, r *http.Request) {
	tenantID, ok := h.scimTenantAndAuth(w, r)
	if !ok {
		return
	}

	subject := chi.URLParam(r, "userID")
	if subject == "" {
		respondError(w, http.StatusBadRequest, "SCIM userID is required")
		return
	}

	if err := h.DB.Users.DeactivateBySubject(r.Context(), tenantID, subject); err != nil {
		h.Logger.Error("Failed to deactivate SCIM user", zap.Error(err))
		respondError(w, http.StatusInternalServerError, "failed to deactivate user")
		return
	}

	respondJSON(w, http.StatusOK, map[string]any{
		"externalId": subject,
		"active":     false,
	})
}

func (h *Handler) scimTenantAndAuth(w http.ResponseWriter, r *http.Request) (string, bool) {
	expected := os.Getenv("SCIM_BEARER_TOKEN")
	if expected == "" {
		h.Logger.Error("SCIM token is not configured")
		respondError(w, http.StatusInternalServerError, "scim is not configured")
		return "", false
	}
	authz := strings.TrimSpace(r.Header.Get("Authorization"))
	if !strings.HasPrefix(strings.ToLower(authz), "bearer ") {
		respondError(w, http.StatusUnauthorized, "missing bearer token")
		return "", false
	}
	given := strings.TrimSpace(authz[len("Bearer "):])
	if given != expected {
		respondError(w, http.StatusUnauthorized, "invalid scim token")
		return "", false
	}

	tenantID := r.URL.Query().Get("tenant_id")
	if tenantID == "" {
		tenantID = strings.TrimSpace(r.Header.Get("X-Tenant-ID"))
	}
	if tenantID == "" {
		respondError(w, http.StatusBadRequest, "tenant_id is required")
		return "", false
	}
	return tenantID, true
}

func scimEmail(req scimUserPayload) string {
	for _, e := range req.Emails {
		if e.Primary && e.Value != "" {
			return strings.ToLower(strings.TrimSpace(e.Value))
		}
	}
	if len(req.Emails) > 0 {
		return strings.ToLower(strings.TrimSpace(req.Emails[0].Value))
	}
	return strings.ToLower(strings.TrimSpace(req.UserName))
}

func scimSubject(req scimUserPayload) string {
	return strings.TrimSpace(req.ExternalID)
}

func scimName(req scimUserPayload) string {
	if strings.TrimSpace(req.DisplayName) != "" {
		return strings.TrimSpace(req.DisplayName)
	}
	full := strings.TrimSpace(strings.TrimSpace(req.Name.GivenName) + " " + strings.TrimSpace(req.Name.FamilyName))
	if full != "" {
		return full
	}
	return strings.TrimSpace(req.UserName)
}
