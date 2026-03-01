package authorization

import "strings"

type teamBinding struct {
	TeamID string
	Role   string
}

var teamRoleActionMatrix = map[string]map[Action]struct{}{
	"viewer":   {ActionRead: {}},
	"operator": {ActionRead: {}, ActionWrite: {}},
	"admin":    {ActionRead: {}, ActionWrite: {}, ActionAdmin: {}},
	"owner":    {ActionRead: {}, ActionWrite: {}, ActionAdmin: {}},
}

// IsTeamAllowed enforces team-scoped role bindings.
// Accepted role formats:
// - team:<team-id>:<role>
// - <role>#<team-id>
// - team/<team-id>/<role>
func IsTeamAllowed(roles []string, teamID string, action Action) bool {
	teamID = normalizeTeamID(teamID)
	if teamID == "" {
		return false
	}

	for _, binding := range parseTeamBindings(roles) {
		if binding.TeamID != teamID {
			continue
		}
		if allowedActions, ok := teamRoleActionMatrix[binding.Role]; ok {
			if _, allowed := allowedActions[action]; allowed {
				return true
			}
			if action != ActionAdmin {
				if _, allowed := allowedActions[ActionAdmin]; allowed {
					return true
				}
			}
		}
	}

	return false
}

func parseTeamBindings(roles []string) []teamBinding {
	bindings := make([]teamBinding, 0, len(roles))
	for _, raw := range roles {
		role := strings.ToLower(strings.TrimSpace(raw))
		if role == "" {
			continue
		}

		if strings.HasPrefix(role, "team:") {
			parts := strings.Split(role, ":")
			if len(parts) == 3 && parts[1] != "" && parts[2] != "" {
				bindings = append(bindings, teamBinding{
					TeamID: normalizeTeamID(parts[1]),
					Role:   strings.TrimSpace(parts[2]),
				})
			}
			continue
		}

		if strings.HasPrefix(role, "team/") {
			parts := strings.Split(role, "/")
			if len(parts) == 3 && parts[1] != "" && parts[2] != "" {
				bindings = append(bindings, teamBinding{
					TeamID: normalizeTeamID(parts[1]),
					Role:   strings.TrimSpace(parts[2]),
				})
			}
			continue
		}

		parts := strings.Split(role, "#")
		if len(parts) == 2 && parts[0] != "" && parts[1] != "" {
			bindings = append(bindings, teamBinding{
				TeamID: normalizeTeamID(parts[1]),
				Role:   strings.TrimSpace(parts[0]),
			})
		}
	}
	return bindings
}

func normalizeTeamID(teamID string) string {
	return strings.ToLower(strings.TrimSpace(teamID))
}
