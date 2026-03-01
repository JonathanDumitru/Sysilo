package storage

import "github.com/sysilo/sysilo/services/api-gateway/internal/db"

// Users is a thin compatibility alias while identity persistence lives in internal/db.
type Users = db.UserRepository
