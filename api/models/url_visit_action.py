from sqlalchemy import Integer, String, DateTime, Index, text
from sqlalchemy.dialects.postgresql import JSONB
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy.sql import func
from db import Base


class UrlVisitAction(Base):
    __tablename__ = "url_visit_actions"

    __table_args__ = (
        Index("ix_url_ip_action_at", "url_id", "ip_address", "action_at"),
        Index("ix_visit_country", "country"),
        Index("ix_visit_city", "city"),
    )

    id: Mapped[int] = mapped_column(primary_key=True)

    url_id: Mapped[int] = mapped_column(Integer, nullable=False, index=True)
    ip_address: Mapped[str] = mapped_column(String(45), nullable=False)

    ip_host: Mapped[str | None] = mapped_column(String)
    ip_provider: Mapped[str | None] = mapped_column(String)

    country: Mapped[str | None] = mapped_column(String)
    city: Mapped[str | None] = mapped_column(String)

    geo_data: Mapped[dict | None] = mapped_column(JSONB)

    user_agent: Mapped[str] = mapped_column(String, nullable=False)
    count: Mapped[int] = mapped_column(
        Integer, nullable=False, default=1, server_default=text("1")
    )

    action_at: Mapped[DateTime] = mapped_column(
        DateTime(timezone=True),
        server_default=func.now(),
        nullable=False,
    )
