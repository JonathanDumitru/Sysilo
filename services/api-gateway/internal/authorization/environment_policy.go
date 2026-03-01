package authorization

import "strings"

type Environment string

const (
	EnvironmentDev     Environment = "dev"
	EnvironmentStaging Environment = "staging"
	EnvironmentProd    Environment = "prod"
)

type Action string

const (
	ActionRead  Action = "read"
	ActionWrite Action = "write"
	ActionAdmin Action = "admin"
)

var environmentAliases = map[string]Environment{
	"dev":         EnvironmentDev,
	"development": EnvironmentDev,
	"staging":     EnvironmentStaging,
	"stage":       EnvironmentStaging,
	"prod":        EnvironmentProd,
	"production":  EnvironmentProd,
}

// roleActionMatrix defines what each role can do in each environment.
var roleActionMatrix = map[string]map[Environment]map[Action]struct{}{
	"viewer": {
		EnvironmentDev:     {ActionRead: {}},
		EnvironmentStaging: {ActionRead: {}},
		EnvironmentProd:    {ActionRead: {}},
	},
	"operator": {
		EnvironmentDev:     {ActionRead: {}, ActionWrite: {}},
		EnvironmentStaging: {ActionRead: {}, ActionWrite: {}},
		EnvironmentProd:    {ActionRead: {}},
	},
	"admin": {
		EnvironmentDev:     {ActionRead: {}, ActionWrite: {}, ActionAdmin: {}},
		EnvironmentStaging: {ActionRead: {}, ActionWrite: {}, ActionAdmin: {}},
		EnvironmentProd:    {ActionRead: {}, ActionWrite: {}, ActionAdmin: {}},
	},
	"owner": {
		EnvironmentDev:     {ActionRead: {}, ActionWrite: {}, ActionAdmin: {}},
		EnvironmentStaging: {ActionRead: {}, ActionWrite: {}, ActionAdmin: {}},
		EnvironmentProd:    {ActionRead: {}, ActionWrite: {}, ActionAdmin: {}},
	},
}

func ParseEnvironment(raw string) (Environment, bool) {
	normalized := strings.ToLower(strings.TrimSpace(raw))
	env, ok := environmentAliases[normalized]
	return env, ok
}

func IsAllowed(roles []string, environment Environment, action Action) bool {
	for _, binding := range parseBindings(roles) {
		if binding.Environment != "" && binding.Environment != environment {
			continue
		}

		allowedActions, ok := roleActionMatrix[binding.Role][environment]
		if !ok {
			continue
		}
		if _, allowed := allowedActions[action]; allowed {
			return true
		}
		// Admin permission includes write/read operations.
		if action != ActionAdmin {
			if _, allowed := allowedActions[ActionAdmin]; allowed {
				return true
			}
		}
	}

	return false
}

// Allow requires both environment RBAC and team-scoped entitlement.
func Allow(roles []string, environment Environment, action Action, teamID string) bool {
	if !IsAllowed(roles, environment, action) {
		return false
	}
	return IsTeamAllowed(roles, teamID, action)
}

// Authorize is an alias for Allow.
func Authorize(roles []string, environment Environment, action Action, teamID string) bool {
	return Allow(roles, environment, action, teamID)
}

type roleBinding struct {
	Role        string
	Environment Environment
}

func parseBindings(roles []string) []roleBinding {
	bindings := make([]roleBinding, 0, len(roles))
	for _, raw := range roles {
		role := strings.ToLower(strings.TrimSpace(raw))
		if role == "" {
			continue
		}

		if scoped, ok := parseScopedRole(role); ok {
			bindings = append(bindings, scoped)
			continue
		}

		bindings = append(bindings, roleBinding{Role: role})
	}
	return bindings
}

func parseScopedRole(raw string) (roleBinding, bool) {
	pairs := [][2]string{
		{":", "prefix"},
		{"@", "suffix"},
		{"/", "prefix"},
	}

	for _, pair := range pairs {
		sep := pair[0]
		parts := strings.Split(raw, sep)
		if len(parts) != 2 {
			continue
		}

		var envRaw, roleRaw string
		if pair[1] == "prefix" {
			envRaw, roleRaw = parts[0], parts[1]
		} else {
			roleRaw, envRaw = parts[0], parts[1]
		}

		environment, ok := ParseEnvironment(envRaw)
		if !ok {
			continue
		}

		return roleBinding{
			Role:        strings.TrimSpace(roleRaw),
			Environment: environment,
		}, true
	}

	return roleBinding{}, false
}
