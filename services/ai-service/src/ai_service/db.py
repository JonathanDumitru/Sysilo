"""Database connections for AI Service."""

from typing import AsyncGenerator

from neo4j import AsyncGraphDatabase, AsyncDriver
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine, async_sessionmaker
import redis.asyncio as redis
import structlog

from ai_service.config import get_settings

settings = get_settings()
logger = structlog.get_logger()

# PostgreSQL
engine = create_async_engine(
    settings.database_url,
    echo=settings.debug,
    pool_size=5,
    max_overflow=10,
)

async_session = async_sessionmaker(
    engine,
    class_=AsyncSession,
    expire_on_commit=False,
)

# Neo4j
neo4j_driver: AsyncDriver | None = None

# Redis
redis_client: redis.Redis | None = None


async def init_db() -> None:
    """Initialize all database connections."""
    global neo4j_driver, redis_client

    # Neo4j
    try:
        neo4j_driver = AsyncGraphDatabase.driver(
            settings.neo4j_uri,
            auth=(settings.neo4j_user, settings.neo4j_password),
        )
        await neo4j_driver.verify_connectivity()
        logger.info("Connected to Neo4j")
    except Exception as e:
        logger.warning("Failed to connect to Neo4j", error=str(e))

    # Redis
    try:
        redis_client = redis.from_url(settings.redis_url)
        await redis_client.ping()
        logger.info("Connected to Redis")
    except Exception as e:
        logger.warning("Failed to connect to Redis", error=str(e))


async def close_db() -> None:
    """Close all database connections."""
    global neo4j_driver, redis_client

    if neo4j_driver:
        await neo4j_driver.close()

    if redis_client:
        await redis_client.close()

    await engine.dispose()


async def get_db() -> AsyncGenerator[AsyncSession, None]:
    """Get PostgreSQL session."""
    async with async_session() as session:
        yield session


async def get_neo4j() -> AsyncDriver | None:
    """Get Neo4j driver."""
    return neo4j_driver


async def get_redis() -> redis.Redis | None:
    """Get Redis client."""
    return redis_client
