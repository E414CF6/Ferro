"use client";

import React, { useCallback, useEffect, useRef, useState } from "react";
import { ArticleFilterInput, TrendsData, WikiArticle } from "@/lib/types";
import { fetchGraphQL, QUERIES } from "@/lib/graphql";
import { useAuth } from "@/lib/auth-context";
import { useToast } from "@/lib/toast-context";
import CreateArticleModal from "@/components/wiki/CreateArticleModal";
import ArticleDetailModal from "@/components/wiki/ArticleDetailModal";
import {
    Compass,
    Eye,
    Globe,
    Hash,
    MapPin,
    Minus,
    Plus,
    RefreshCw,
    Search,
    Sparkles,
    TrendingDown,
    TrendingUp,
    Zap,
    X,
} from "lucide-react";

const CATEGORIES = [
    { id: "all", label: "전체" },
    { id: "장소/명소", label: "장소/명소" },
    { id: "역사/문화", label: "역사/문화" },
    { id: "기술/개발", label: "기술/개발" },
    { id: "자연/지리", label: "자연/지리" },
    { id: "맛집/카페", label: "맛집/카페" },
];

export default function WikiMapPage() {
    const { user } = useAuth();
    const { showToast } = useToast();

    // Data states
    const [articles, setArticles] = useState<WikiArticle[]>([]);
    const [trends, setTrends] = useState<TrendsData | null>(null);
    const [loadingArticles, setLoadingArticles] = useState(true);
    const [loadingTrends, setLoadingTrends] = useState(false);

    // Filters
    const [searchQuery, setSearchQuery] = useState("");
    const [selectedCategory, setSelectedCategory] = useState("all");
    const [selectedTag, setSelectedTag] = useState<string | null>(null);

    // Modals & Selected items
    const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [clickedCoords, setClickedCoords] = useState<{ lat: number; lng: number } | null>(null);

    // Map Viewport state (Center Seoul: 37.5665, 126.9780, Zoom scale)
    const [centerLat, setCenterLat] = useState(37.5665);
    const [centerLng, setCenterLng] = useState(126.9780);
    const [zoomScale, setZoomScale] = useState(1);
    const [isDragging, setIsDragging] = useState(false);
    const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

    const mapContainerRef = useRef<HTMLDivElement | null>(null);

    // Load Articles with filter
    const loadArticles = useCallback(async () => {
        setLoadingArticles(true);
        try {
            const filter: ArticleFilterInput = {};
            if (searchQuery.trim()) filter.q = searchQuery.trim();
            if (selectedTag) filter.tag = selectedTag;

            const data = await fetchGraphQL<{ articles: WikiArticle[] }>(
                QUERIES.WIKI_ARTICLES,
                { filter: Object.keys(filter).length > 0 ? filter : undefined }
            );

            if (data?.articles) {
                let filtered = data.articles;
                if (selectedCategory !== "all") {
                    filtered = filtered.filter((a) => a.category === selectedCategory);
                }
                setArticles(filtered);
            }
        } catch (err) {
            console.error("Failed to load wiki articles:", err);
        } finally {
            setLoadingArticles(false);
        }
    }, [searchQuery, selectedCategory, selectedTag]);

    // Load Trends
    const loadTrends = useCallback(async (forceRefresh = false) => {
        setLoadingTrends(true);
        try {
            const data = await fetchGraphQL<{ wikiTrends: TrendsData }>(
                QUERIES.WIKI_TRENDS,
                { forceRefresh }
            );
            if (data?.wikiTrends) {
                setTrends(data.wikiTrends);
            }
        } catch (err) {
            console.error("Failed to load wiki trends:", err);
        } finally {
            setLoadingTrends(false);
        }
    }, []);

    useEffect(() => {
        loadArticles();
    }, [loadArticles]);

    useEffect(() => {
        loadTrends();
        const interval = setInterval(() => loadTrends(false), 20000);
        return () => clearInterval(interval);
    }, [loadTrends]);

    // Pan & Zoom Map handlers
    const handleZoomIn = () => setZoomScale((prev) => Math.min(prev * 1.3, 5));
    const handleZoomOut = () => setZoomScale((prev) => Math.max(prev / 1.3, 0.4));
    const handleResetView = () => {
        setCenterLat(37.5665);
        setCenterLng(126.9780);
        setZoomScale(1);
    };

    // Map drag navigation
    const handleMouseDown = (e: React.MouseEvent) => {
        if ((e.target as HTMLElement).closest(".map-pin-btn")) return;
        setIsDragging(true);
        setDragStart({ x: e.clientX, y: e.clientY });
    };

    const handleMouseMove = (e: React.MouseEvent) => {
        if (!isDragging) return;
        const dx = e.clientX - dragStart.x;
        const dy = e.clientY - dragStart.y;
        setDragStart({ x: e.clientX, y: e.clientY });

        // Convert pixel drag to approximate lat/lng delta
        const degPerPixel = 0.002 / zoomScale;
        setCenterLng((prev) => prev - dx * degPerPixel);
        setCenterLat((prev) => prev + dy * degPerPixel);
    };

    const handleMouseUp = () => setIsDragging(false);

    // Map click to place coordinate pin
    const handleMapClick = (e: React.MouseEvent) => {
        if ((e.target as HTMLElement).closest(".map-pin-btn")) return;
        if (!mapContainerRef.current) return;

        const rect = mapContainerRef.current.getBoundingClientRect();
        const px = e.clientX - rect.left - rect.width / 2;
        const py = e.clientY - rect.top - rect.height / 2;

        const degPerPixel = 0.002 / zoomScale;
        const clickedLng = centerLng + px * degPerPixel;
        const clickedLat = centerLat - py * degPerPixel;

        setClickedCoords({
            lat: parseFloat(clickedLat.toFixed(5)),
            lng: parseFloat(clickedLng.toFixed(5)),
        });
    };

    // Convert lat/lng to viewport (x, y)
    const getPinPosition = (lat: number, lng: number) => {
        if (!mapContainerRef.current) return { x: 0, y: 0 };
        const rect = mapContainerRef.current.getBoundingClientRect();
        const degPerPixel = 0.002 / zoomScale;

        const x = rect.width / 2 + (lng - centerLng) / degPerPixel;
        const y = rect.height / 2 - (lat - centerLat) / degPerPixel;
        return { x, y };
    };

    return (
        <div style={{ display: "flex", flexDirection: "column", height: "calc(100vh - 60px)", position: "relative", overflow: "hidden" }}>
            {/* Top Toolbar */}
            <div
                style={{
                    padding: "12px 20px",
                    backgroundColor: "var(--bg-surface)",
                    borderBottom: "1px solid var(--border-subtle)",
                    display: "flex",
                    flexWrap: "wrap",
                    alignItems: "center",
                    justifyContent: "space-between",
                    gap: "12px",
                    zIndex: 10,
                }}
            >
                <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
                    <div
                        style={{
                            width: 36,
                            height: 36,
                            borderRadius: "10px",
                            background: "linear-gradient(135deg, #0284c7 0%, #a855f7 100%)",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "center",
                            color: "#fff",
                            boxShadow: "0 0 14px rgba(56, 189, 248, 0.3)",
                        }}
                    >
                        <Globe size={20} />
                    </div>
                    <div>
                        <h1 style={{ fontSize: "17px", fontWeight: 800, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: "6px" }}>
                            위키 맵 <span style={{ fontSize: "12px", color: "var(--accent-primary)", fontWeight: 600 }}>wMap</span>
                        </h1>
                        <p style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                            지리 공간 기반의 실시간 협업형 지식 위키
                        </p>
                    </div>
                </div>

                {/* Search Bar */}
                <div style={{ position: "relative", minWidth: "240px", flex: "1 1 auto", maxWidth: "400px" }}>
                    <Search
                        size={15}
                        style={{
                            position: "absolute",
                            left: 12,
                            top: "50%",
                            transform: "translateY(-50%)",
                            color: "var(--text-muted)",
                        }}
                    />
                    <input
                        type="text"
                        placeholder="위키 문서, 지역, 키워드 검색..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        style={{
                            width: "100%",
                            padding: "8px 32px 8px 36px",
                            borderRadius: "var(--radius-full)",
                            backgroundColor: "var(--bg-input)",
                            border: "1px solid var(--border-subtle)",
                            color: "var(--text-primary)",
                            fontSize: "13px",
                            outline: "none",
                        }}
                    />
                    {searchQuery && (
                        <button
                            type="button"
                            onClick={() => setSearchQuery("")}
                            style={{ position: "absolute", right: 10, top: "50%", transform: "translateY(-50%)", color: "var(--text-muted)" }}
                        >
                            <X size={13} />
                        </button>
                    )}
                </div>

                {/* Category Pills & Action */}
                <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                    <div style={{ display: "flex", gap: "4px", overflowX: "auto", scrollbarWidth: "none" }}>
                        {CATEGORIES.map((cat) => (
                            <button
                                key={cat.id}
                                onClick={() => setSelectedCategory(cat.id)}
                                className={`header-tab-pill ${selectedCategory === cat.id ? "active" : ""}`}
                                style={{ fontSize: "12px", padding: "4px 10px", flexShrink: 0 }}
                            >
                                {cat.label}
                            </button>
                        ))}
                    </div>

                    <button
                        onClick={() => {
                            setClickedCoords(null);
                            setShowCreateModal(true);
                        }}
                        className="btn-primary"
                        style={{ padding: "6px 14px", fontSize: "12px", whiteSpace: "nowrap" }}
                    >
                        <Plus size={14} /> 새 위키 작성
                    </button>
                </div>
            </div>

            {/* Main Interactive Map & Trending Split View */}
            <div style={{ display: "flex", flex: 1, position: "relative", overflow: "hidden" }}>
                {/* Map Canvas Container */}
                <div
                    ref={mapContainerRef}
                    onMouseDown={handleMouseDown}
                    onMouseMove={handleMouseMove}
                    onMouseUp={handleMouseUp}
                    onClick={handleMapClick}
                    style={{
                        flex: 1,
                        position: "relative",
                        backgroundColor: "#0b101b",
                        cursor: isDragging ? "grabbing" : "crosshair",
                        userSelect: "none",
                        overflow: "hidden",
                    }}
                >
                    {/* Geospatial Coordinate Grid Canvas */}
                    <div
                        style={{
                            position: "absolute",
                            inset: 0,
                            backgroundImage: `
                                linear-gradient(rgba(56, 189, 248, 0.07) 1px, transparent 1px),
                                linear-gradient(90deg, rgba(56, 189, 248, 0.07) 1px, transparent 1px)
                            `,
                            backgroundSize: `${40 * zoomScale}px ${40 * zoomScale}px`,
                            backgroundPosition: "center center",
                            pointerEvents: "none",
                        }}
                    />

                    {/* Central Crosshair & Scale readout */}
                    <div
                        style={{
                            position: "absolute",
                            bottom: 16,
                            left: 16,
                            backgroundColor: "rgba(15, 23, 42, 0.85)",
                            backdropFilter: "blur(6px)",
                            border: "1px solid var(--border-subtle)",
                            borderRadius: "var(--radius-md)",
                            padding: "6px 12px",
                            fontSize: "11px",
                            color: "var(--text-secondary)",
                            display: "flex",
                            alignItems: "center",
                            gap: "8px",
                            pointerEvents: "none",
                        }}
                    >
                        <Compass size={13} color="var(--accent-primary)" />
                        <span>
                            중심: {centerLat.toFixed(4)}°N, {centerLng.toFixed(4)}°E (줌: {zoomScale.toFixed(1)}x)
                        </span>
                    </div>

                    {/* Zoom / Map Controls */}
                    <div
                        style={{
                            position: "absolute",
                            top: 16,
                            right: 16,
                            display: "flex",
                            flexDirection: "column",
                            gap: "6px",
                            zIndex: 5,
                        }}
                    >
                        <button
                            onClick={handleZoomIn}
                            title="확대"
                            className="btn-secondary"
                            style={{ padding: "8px", borderRadius: "8px" }}
                        >
                            <Plus size={16} />
                        </button>
                        <button
                            onClick={handleZoomOut}
                            title="축소"
                            className="btn-secondary"
                            style={{ padding: "8px", borderRadius: "8px" }}
                        >
                            <Minus size={16} />
                        </button>
                        <button
                            onClick={handleResetView}
                            title="서울 중심으로 리셋"
                            className="btn-secondary"
                            style={{ padding: "8px", borderRadius: "8px" }}
                        >
                            <Compass size={16} />
                        </button>
                    </div>

                    {/* Placed Coordinates Marker (from click) */}
                    {clickedCoords && (
                        <div
                            style={{
                                position: "absolute",
                                left: `${getPinPosition(clickedCoords.lat, clickedCoords.lng).x}px`,
                                top: `${getPinPosition(clickedCoords.lat, clickedCoords.lng).y}px`,
                                transform: "translate(-50%, -100%)",
                                display: "flex",
                                flexDirection: "column",
                                alignItems: "center",
                                pointerEvents: "auto",
                                zIndex: 8,
                            }}
                            onClick={(e) => e.stopPropagation()}
                        >
                            <div
                                style={{
                                    backgroundColor: "rgba(15, 23, 42, 0.95)",
                                    border: "1px solid var(--accent-primary)",
                                    borderRadius: "var(--radius-md)",
                                    padding: "6px 10px",
                                    marginBottom: "4px",
                                    boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
                                    textAlign: "center",
                                }}
                            >
                                <div style={{ fontSize: "11px", fontWeight: 700, color: "var(--accent-primary)" }}>
                                    선택 좌표: {clickedCoords.lat}, {clickedCoords.lng}
                                </div>
                                <button
                                    onClick={() => setShowCreateModal(true)}
                                    className="btn-primary"
                                    style={{ marginTop: "4px", padding: "4px 8px", fontSize: "11px", width: "100%" }}
                                >
                                    <Plus size={11} /> 이 위치에 문서 작성
                                </button>
                            </div>
                            <div style={{ color: "var(--accent-primary)", filter: "drop-shadow(0 0 6px rgba(56, 189, 248, 0.8))" }}>
                                <MapPin size={28} fill="var(--accent-primary)" color="#fff" />
                            </div>
                        </div>
                    )}

                    {/* Article Pins */}
                    {articles.map((art) => {
                        const { x, y } = getPinPosition(art.latitude, art.longitude);
                        return (
                            <div
                                key={art.id}
                                className="map-pin-btn"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    setSelectedSlug(art.slug);
                                }}
                                style={{
                                    position: "absolute",
                                    left: `${x}px`,
                                    top: `${y}px`,
                                    transform: "translate(-50%, -100%)",
                                    cursor: "pointer",
                                    display: "flex",
                                    flexDirection: "column",
                                    alignItems: "center",
                                    transition: "transform 0.15s ease",
                                    zIndex: 6,
                                }}
                                onMouseEnter={(e) => (e.currentTarget.style.transform = "translate(-50%, -105%) scale(1.1)")}
                                onMouseLeave={(e) => (e.currentTarget.style.transform = "translate(-50%, -100%) scale(1)")}
                            >
                                <div
                                    style={{
                                        backgroundColor: "rgba(15, 23, 42, 0.9)",
                                        border: "1px solid var(--border-subtle)",
                                        borderRadius: "var(--radius-full)",
                                        padding: "2px 8px",
                                        fontSize: "11px",
                                        fontWeight: 700,
                                        color: "var(--text-primary)",
                                        whiteSpace: "nowrap",
                                        marginBottom: "2px",
                                        boxShadow: "0 2px 8px rgba(0,0,0,0.4)",
                                    }}
                                >
                                    {art.title}
                                </div>
                                <div
                                    style={{
                                        color: "#ec4899",
                                        filter: "drop-shadow(0 0 5px rgba(236, 72, 153, 0.7))",
                                    }}
                                >
                                    <MapPin size={24} fill="#ec4899" color="#fff" />
                                </div>
                            </div>
                        );
                    })}
                </div>

                {/* Right Insights Sidebar: Trending Articles & Tags */}
                <div
                    style={{
                        width: "340px",
                        backgroundColor: "var(--bg-surface)",
                        borderLeft: "1px solid var(--border-subtle)",
                        display: "flex",
                        flexDirection: "column",
                        overflowY: "auto",
                        flexShrink: 0,
                    }}
                >
                    {/* Header */}
                    <div
                        style={{
                            padding: "14px 16px",
                            borderBottom: "1px solid var(--border-subtle)",
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "space-between",
                        }}
                    >
                        <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                            <TrendingUp size={16} color="#ec4899" />
                            <span style={{ fontSize: "14px", fontWeight: 800, color: "var(--text-primary)" }}>
                                실시간 위키 트렌드
                            </span>
                        </div>

                        <button
                            onClick={() => loadTrends(true)}
                            disabled={loadingTrends}
                            title="트렌드 갱신"
                            style={{ color: "var(--text-muted)", padding: "4px" }}
                        >
                            <RefreshCw size={13} style={{ animation: loadingTrends ? "spin 1s linear infinite" : "none" }} />
                        </button>
                    </div>

                    {/* Top 10 Trending Articles */}
                    <div style={{ padding: "12px 14px", borderBottom: "1px solid var(--border-subtle)" }}>
                        <div style={{ fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "8px", display: "flex", alignItems: "center", gap: "4px" }}>
                            <Zap size={13} color="var(--accent-primary)" /> 인기 문서 TOP 10
                        </div>

                        {!trends || trends.articles.length === 0 ? (
                            <div style={{ padding: "12px 0", textAlign: "center", fontSize: "12px", color: "var(--text-muted)" }}>
                                집계된 인기 문서가 없습니다.
                            </div>
                        ) : (
                            <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                                {trends.articles.map((item) => {
                                    const rankBadgeColor =
                                        item.rank === 1 ? "#eab308" : item.rank === 2 ? "#94a3b8" : item.rank === 3 ? "#b45309" : "var(--text-muted)";

                                    return (
                                        <div
                                            key={item.id}
                                            onClick={() => {
                                                setSelectedSlug(item.slug);
                                                setCenterLat(item.latitude);
                                                setCenterLng(item.longitude);
                                            }}
                                            style={{
                                                padding: "8px 10px",
                                                borderRadius: "var(--radius-md)",
                                                backgroundColor: "var(--bg-input)",
                                                display: "flex",
                                                alignItems: "center",
                                                justifyContent: "space-between",
                                                cursor: "pointer",
                                                transition: "all var(--transition-fast)",
                                            }}
                                            onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-surface-hover)")}
                                            onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "var(--bg-input)")}
                                        >
                                            <div style={{ display: "flex", alignItems: "center", gap: "8px", minWidth: 0 }}>
                                                <span style={{ fontSize: "13px", fontWeight: 900, color: rankBadgeColor, width: "16px", textAlign: "center" }}>
                                                    {item.rank}
                                                </span>
                                                <div style={{ minWidth: 0 }}>
                                                    <div
                                                        style={{
                                                            fontSize: "13px",
                                                            fontWeight: 700,
                                                            color: "var(--text-primary)",
                                                            whiteSpace: "nowrap",
                                                            overflow: "hidden",
                                                            textOverflow: "ellipsis",
                                                            maxWidth: "180px",
                                                        }}
                                                    >
                                                        {item.title}
                                                    </div>
                                                    <div style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                        조회수 {item.views}회 · {item.category}
                                                    </div>
                                                </div>
                                            </div>

                                            {/* Rank change indicator */}
                                            <div>
                                                {item.change === "UP" && (
                                                    <span style={{ fontSize: "11px", fontWeight: 700, color: "#10b981", display: "inline-flex", alignItems: "center" }}>
                                                        <TrendingUp size={12} /> {item.changeAmount ? `+${item.changeAmount}` : ""}
                                                    </span>
                                                )}
                                                {item.change === "DOWN" && (
                                                    <span style={{ fontSize: "11px", fontWeight: 700, color: "#f43f5e", display: "inline-flex", alignItems: "center" }}>
                                                        <TrendingDown size={12} /> {item.changeAmount ? `-${item.changeAmount}` : ""}
                                                    </span>
                                                )}
                                                {item.change === "NEW" && (
                                                    <span style={{ fontSize: "10px", fontWeight: 800, padding: "2px 5px", borderRadius: "4px", backgroundColor: "rgba(234, 179, 8, 0.2)", color: "#eab308" }}>
                                                        NEW
                                                    </span>
                                                )}
                                                {item.change === "SAME" && (
                                                    <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>-</span>
                                                )}
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        )}
                    </div>

                    {/* Trending Tags */}
                    <div style={{ padding: "12px 14px", borderBottom: "1px solid var(--border-subtle)" }}>
                        <div style={{ fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "8px", display: "flex", alignItems: "center", gap: "4px" }}>
                            <Hash size={13} color="#a855f7" /> 인기 공간 태그
                        </div>

                        {!trends || trends.tags.length === 0 ? (
                            <div style={{ fontSize: "12px", color: "var(--text-muted)" }}>집계된 태그가 없습니다.</div>
                        ) : (
                            <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
                                {trends.tags.map((t) => (
                                    <button
                                        key={t.tag}
                                        onClick={() => {
                                            if (selectedTag === t.tag) {
                                                setSelectedTag(null);
                                            } else {
                                                setSelectedTag(t.tag);
                                            }
                                        }}
                                        style={{
                                            fontSize: "12px",
                                            fontWeight: 600,
                                            padding: "4px 10px",
                                            borderRadius: "var(--radius-full)",
                                            backgroundColor: selectedTag === t.tag ? "var(--accent-purple)" : "rgba(168, 85, 247, 0.12)",
                                            color: selectedTag === t.tag ? "#fff" : "var(--accent-purple)",
                                            border: "none",
                                            cursor: "pointer",
                                            display: "inline-flex",
                                            alignItems: "center",
                                            gap: "4px",
                                        }}
                                    >
                                        #{t.tag} <span style={{ fontSize: "10px", opacity: 0.8 }}>({t.count})</span>
                                    </button>
                                ))}
                            </div>
                        )}
                    </div>

                    {/* Articles Catalog List */}
                    <div style={{ padding: "12px 14px", flex: 1 }}>
                        <div style={{ fontSize: "12px", fontWeight: 700, color: "var(--text-secondary)", marginBottom: "8px" }}>
                            등록된 위키 문서 ({articles.length}건)
                        </div>

                        {loadingArticles ? (
                            <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-muted)", fontSize: "12px" }}>
                                문서를 불러오는 중...
                            </div>
                        ) : articles.length === 0 ? (
                            <div style={{ padding: "20px 0", textAlign: "center", color: "var(--text-muted)", fontSize: "12px" }}>
                                조건에 일치하는 위키 문서가 없습니다.
                            </div>
                        ) : (
                            <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                                {articles.map((art) => (
                                    <div
                                        key={art.id}
                                        onClick={() => {
                                            setSelectedSlug(art.slug);
                                            setCenterLat(art.latitude);
                                            setCenterLng(art.longitude);
                                        }}
                                        style={{
                                            padding: "10px",
                                            borderRadius: "var(--radius-md)",
                                            backgroundColor: "var(--bg-input)",
                                            border: "1px solid var(--border-subtle)",
                                            cursor: "pointer",
                                            transition: "all var(--transition-fast)",
                                        }}
                                        onMouseEnter={(e) => (e.currentTarget.style.borderColor = "var(--accent-primary)")}
                                        onMouseLeave={(e) => (e.currentTarget.style.borderColor = "var(--border-subtle)")}
                                    >
                                        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "4px" }}>
                                            <span style={{ fontSize: "13px", fontWeight: 800, color: "var(--text-primary)" }}>
                                                {art.title}
                                            </span>
                                            <span style={{ fontSize: "11px", color: "var(--text-muted)" }}>
                                                {art.category}
                                            </span>
                                        </div>
                                        {art.summary && (
                                            <p style={{ fontSize: "11px", color: "var(--text-secondary)", lineHeight: "1.4", marginBottom: "6px" }}>
                                                {art.summary}
                                            </p>
                                        )}
                                        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", fontSize: "10px", color: "var(--text-muted)" }}>
                                            <span>작성: {art.author}</span>
                                            <span style={{ display: "inline-flex", alignItems: "center", gap: "3px" }}>
                                                <Eye size={10} /> {art.views}
                                            </span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* Modals */}
            {showCreateModal && (
                <CreateArticleModal
                    initialCoords={clickedCoords}
                    onClose={() => setShowCreateModal(false)}
                    onCreated={(newArt) => {
                        setArticles((prev) => [newArt, ...prev]);
                        setCenterLat(newArt.latitude);
                        setCenterLng(newArt.longitude);
                    }}
                />
            )}

            {selectedSlug && (
                <ArticleDetailModal
                    slug={selectedSlug}
                    onClose={() => setSelectedSlug(null)}
                    onArticleUpdated={loadArticles}
                    onArticleDeleted={loadArticles}
                />
            )}
        </div>
    );
}
