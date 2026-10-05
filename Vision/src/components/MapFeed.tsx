"use client";

import React, { useCallback, useEffect, useRef, useState } from "react";
import { ArticleFilterInput, WikiArticle } from "@/lib/types";
import { fetchGraphQL, QUERIES } from "@/lib/graphql";
import { useAuth } from "@/lib/auth-context";
import CreateArticleModal from "@/components/wiki/CreateArticleModal";
import ArticleDetailModal from "@/components/wiki/ArticleDetailModal";
import {
    Compass,
    Eye,
    Globe,
    MapPin,
    Minus,
    Plus,
    RefreshCw,
    Search,
    Sparkles,
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

export default function MapFeed() {
    const { user } = useAuth();

    // Data states
    const [articles, setArticles] = useState<WikiArticle[]>([]);
    const [loadingArticles, setLoadingArticles] = useState(true);

    // Filters
    const [searchQuery, setSearchQuery] = useState("");
    const [selectedCategory, setSelectedCategory] = useState("all");

    // Modals & Selected items
    const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [clickedCoords, setClickedCoords] = useState<{ lat: number; lng: number } | null>(null);

    // Map Viewport state (Center Seoul: 37.5665, 126.9780)
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
            console.error("Failed to load map articles:", err);
        } finally {
            setLoadingArticles(false);
        }
    }, [searchQuery, selectedCategory]);

    useEffect(() => {
        loadArticles();
    }, [loadArticles]);

    // Zoom controls
    const handleZoomIn = () => setZoomScale((prev) => Math.min(prev * 1.3, 4));
    const handleZoomOut = () => setZoomScale((prev) => Math.max(prev / 1.3, 0.5));
    const handleResetCenter = () => {
        setCenterLat(37.5665);
        setCenterLng(126.9780);
        setZoomScale(1);
    };

    // Pan / Drag handling
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
        <div style={{ display: "flex", flexDirection: "column", minHeight: "600px" }}>
            {/* Filter & Search Bar */}
            <div
                style={{
                    padding: "12px 16px",
                    backgroundColor: "var(--bg-surface)",
                    borderBottom: "1px solid var(--border-subtle)",
                    display: "flex",
                    flexDirection: "column",
                    gap: "10px",
                }}
            >
                {/* Search Input & Action */}
                <div style={{ display: "flex", gap: "10px", alignItems: "center" }}>
                    <div style={{ position: "relative", flex: 1 }}>
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
                            placeholder="지도 위키 장소, 명소, 키워드 검색..."
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
                                style={{
                                    position: "absolute",
                                    right: 10,
                                    top: "50%",
                                    transform: "translateY(-50%)",
                                    color: "var(--text-muted)",
                                }}
                            >
                                <X size={14} />
                            </button>
                        )}
                    </div>

                    {user && (
                        <button
                            type="button"
                            onClick={() => {
                                setClickedCoords(null);
                                setShowCreateModal(true);
                            }}
                            className="btn-primary"
                            style={{
                                padding: "8px 14px",
                                fontSize: "12px",
                                fontWeight: 700,
                                borderRadius: "var(--radius-full)",
                                whiteSpace: "nowrap",
                            }}
                        >
                            <Plus size={14} /> 새 장소 등록
                        </button>
                    )}
                </div>

                {/* Categories */}
                <div
                    style={{
                        display: "flex",
                        gap: "6px",
                        overflowX: "auto",
                        scrollbarWidth: "none",
                        paddingBottom: "2px",
                    }}
                >
                    {CATEGORIES.map((cat) => (
                        <button
                            key={cat.id}
                            type="button"
                            onClick={() => setSelectedCategory(cat.id)}
                            style={{
                                padding: "5px 12px",
                                borderRadius: "var(--radius-full)",
                                fontSize: "12px",
                                fontWeight: 600,
                                whiteSpace: "nowrap",
                                border: "1px solid var(--border-subtle)",
                                backgroundColor:
                                    selectedCategory === cat.id
                                        ? "var(--accent-primary)"
                                        : "var(--bg-surface-elevated)",
                                color:
                                    selectedCategory === cat.id
                                        ? "#fff"
                                        : "var(--text-secondary)",
                                cursor: "pointer",
                                transition: "all 0.15s ease",
                            }}
                        >
                            {cat.label}
                        </button>
                    ))}
                </div>
            </div>

            {/* Interactive Map Canvas Container */}
            <div
                ref={mapContainerRef}
                onMouseDown={handleMouseDown}
                onMouseMove={handleMouseMove}
                onMouseUp={handleMouseUp}
                onClick={handleMapClick}
                style={{
                    position: "relative",
                    width: "100%",
                    height: "380px",
                    backgroundColor: "#070c18",
                    backgroundImage: `
                        radial-gradient(circle at 50% 50%, rgba(56, 189, 248, 0.06) 0%, transparent 60%),
                        linear-gradient(rgba(255, 255, 255, 0.04) 1px, transparent 1px),
                        linear-gradient(90deg, rgba(255, 255, 255, 0.04) 1px, transparent 1px)
                    `,
                    backgroundSize: "100% 100%, 40px 40px, 40px 40px",
                    overflow: "hidden",
                    cursor: isDragging ? "grabbing" : "grab",
                    borderBottom: "1px solid var(--border-subtle)",
                }}
            >
                {/* Floating Map Zoom & Navigation Controls */}
                <div
                    style={{
                        position: "absolute",
                        right: 14,
                        bottom: 14,
                        display: "flex",
                        flexDirection: "column",
                        gap: "6px",
                        zIndex: 10,
                    }}
                >
                    <button
                        type="button"
                        onClick={(e) => {
                            e.stopPropagation();
                            handleZoomIn();
                        }}
                        className="btn-secondary"
                        style={{ width: 34, height: 34, padding: 0, justifyContent: "center", borderRadius: "8px" }}
                        title="확대"
                    >
                        <Plus size={16} />
                    </button>
                    <button
                        type="button"
                        onClick={(e) => {
                            e.stopPropagation();
                            handleZoomOut();
                        }}
                        className="btn-secondary"
                        style={{ width: 34, height: 34, padding: 0, justifyContent: "center", borderRadius: "8px" }}
                        title="축소"
                    >
                        <Minus size={16} />
                    </button>
                    <button
                        type="button"
                        onClick={(e) => {
                            e.stopPropagation();
                            handleResetCenter();
                        }}
                        className="btn-secondary"
                        style={{ width: 34, height: 34, padding: 0, justifyContent: "center", borderRadius: "8px" }}
                        title="서울 중심 복귀"
                    >
                        <Compass size={16} />
                    </button>
                </div>

                {/* Coordinate Badge */}
                <div
                    style={{
                        position: "absolute",
                        left: 14,
                        bottom: 14,
                        backgroundColor: "rgba(11, 15, 25, 0.8)",
                        backdropFilter: "blur(6px)",
                        border: "1px solid var(--border-subtle)",
                        borderRadius: "var(--radius-sm)",
                        padding: "4px 10px",
                        fontSize: "11px",
                        color: "var(--text-muted)",
                        fontFamily: "var(--font-mono)",
                        pointerEvents: "none",
                        zIndex: 5,
                    }}
                >
                    {centerLat.toFixed(4)}°N, {centerLng.toFixed(4)}°E (x{zoomScale.toFixed(1)})
                </div>

                {/* Render Article Map Pins */}
                {articles.map((art) => {
                    if (art.latitude == null || art.longitude == null) return null;
                    const pos = getPinPosition(art.latitude, art.longitude);

                    // Skip pins outside visible viewport
                    if (pos.x < -60 || pos.x > 800 || pos.y < -60 || pos.y > 500) {
                        return null;
                    }

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
                                left: `${pos.x}px`,
                                top: `${pos.y}px`,
                                transform: "translate(-50%, -100%)",
                                display: "flex",
                                flexDirection: "column",
                                alignItems: "center",
                                cursor: "pointer",
                                zIndex: 12,
                                transition: "transform 0.15s ease",
                            }}
                        >
                            <div
                                style={{
                                    backgroundColor: "rgba(11, 15, 25, 0.9)",
                                    border: "1px solid rgba(56, 189, 248, 0.4)",
                                    borderRadius: "16px",
                                    padding: "3px 8px",
                                    display: "flex",
                                    alignItems: "center",
                                    gap: "4px",
                                    boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
                                    maxWidth: "140px",
                                }}
                            >
                                <span style={{ width: 6, height: 6, borderRadius: "50%", backgroundColor: "#38bdf8" }} />
                                <span
                                    style={{
                                        fontSize: "11px",
                                        fontWeight: 700,
                                        color: "#fff",
                                        whiteSpace: "nowrap",
                                        overflow: "hidden",
                                        textOverflow: "ellipsis",
                                    }}
                                >
                                    {art.title}
                                </span>
                            </div>
                            <MapPin size={22} color="#38bdf8" style={{ marginTop: "-2px", filter: "drop-shadow(0 2px 4px rgba(0,0,0,0.5))" }} />
                        </div>
                    );
                })}

                {/* Clicked Coordinate Pin Preview */}
                {clickedCoords && (
                    (() => {
                        const pos = getPinPosition(clickedCoords.lat, clickedCoords.lng);
                        return (
                            <div
                                style={{
                                    position: "absolute",
                                    left: `${pos.x}px`,
                                    top: `${pos.y}px`,
                                    transform: "translate(-50%, -100%)",
                                    zIndex: 15,
                                    display: "flex",
                                    flexDirection: "column",
                                    alignItems: "center",
                                }}
                            >
                                <div
                                    style={{
                                        backgroundColor: "rgba(168, 85, 247, 0.95)",
                                        color: "#fff",
                                        fontSize: "11px",
                                        padding: "4px 8px",
                                        borderRadius: "6px",
                                        fontWeight: 700,
                                        whiteSpace: "nowrap",
                                        marginBottom: "2px",
                                        cursor: "pointer",
                                    }}
                                    onClick={() => setShowCreateModal(true)}
                                >
                                    + 이 위치에 등록
                                </div>
                                <MapPin size={26} color="#a855f7" />
                            </div>
                        );
                    })()
                )}
            </div>

            {/* Articles Cards List Below Map */}
            <div style={{ padding: "16px 20px" }}>
                <div
                    style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                        marginBottom: "14px",
                    }}
                >
                    <h3 style={{ fontSize: "15px", fontWeight: 800, color: "var(--text-primary)" }}>
                        지도 위키 장소 ({articles.length}건)
                    </h3>
                </div>

                {loadingArticles ? (
                    <div style={{ padding: "30px 0", textAlign: "center", color: "var(--text-muted)", fontSize: "13px" }}>
                        장소 목록을 불러오는 중...
                    </div>
                ) : articles.length === 0 ? (
                    <div style={{ padding: "30px 0", textAlign: "center", color: "var(--text-muted)", fontSize: "13px" }}>
                        해당 분류의 장소 위키가 없습니다.
                    </div>
                ) : (
                    <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(260px, 1fr))", gap: "12px" }}>
                        {articles.map((art) => (
                            <div
                                key={art.id}
                                onClick={() => setSelectedSlug(art.slug)}
                                style={{
                                    backgroundColor: "var(--bg-surface)",
                                    border: "1px solid var(--border-subtle)",
                                    borderRadius: "var(--radius-md)",
                                    padding: "14px",
                                    cursor: "pointer",
                                    transition: "all 0.2s ease",
                                }}
                                onMouseEnter={(e) => {
                                    e.currentTarget.style.borderColor = "var(--border-hover)";
                                    e.currentTarget.style.transform = "translateY(-2px)";
                                }}
                                onMouseLeave={(e) => {
                                    e.currentTarget.style.borderColor = "var(--border-subtle)";
                                    e.currentTarget.style.transform = "none";
                                }}
                            >
                                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "6px" }}>
                                    <span
                                        style={{
                                            fontSize: "11px",
                                            fontWeight: 700,
                                            color: "var(--accent-primary)",
                                            backgroundColor: "rgba(56, 189, 248, 0.12)",
                                            padding: "2px 8px",
                                            borderRadius: "12px",
                                        }}
                                    >
                                        {art.category}
                                    </span>
                                    <span style={{ fontSize: "11px", color: "var(--text-muted)", display: "flex", alignItems: "center", gap: "3px" }}>
                                        <Eye size={12} /> {art.views}
                                    </span>
                                </div>
                                <h4 style={{ fontSize: "14px", fontWeight: 700, color: "var(--text-primary)", marginBottom: "4px" }}>
                                    {art.title}
                                </h4>
                                <p
                                    style={{
                                        fontSize: "12px",
                                        color: "var(--text-secondary)",
                                        lineHeight: 1.4,
                                        display: "-webkit-box",
                                        WebkitLineClamp: 2,
                                        WebkitBoxOrient: "vertical",
                                        overflow: "hidden",
                                        marginBottom: "8px",
                                    }}
                                >
                                    {art.content}
                                </p>
                                {art.latitude != null && art.longitude != null && (
                                    <div style={{ fontSize: "11px", color: "var(--text-muted)", display: "flex", alignItems: "center", gap: "4px" }}>
                                        <MapPin size={12} color="#38bdf8" />
                                        <span>{art.latitude.toFixed(3)}, {art.longitude.toFixed(3)}</span>
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                )}
            </div>

            {/* Modals */}
            {selectedSlug && (
                <ArticleDetailModal
                    slug={selectedSlug}
                    onClose={() => setSelectedSlug(null)}
                    onArticleUpdated={loadArticles}
                />
            )}

            {showCreateModal && (
                <CreateArticleModal
                    initialCoords={clickedCoords || undefined}
                    onClose={() => {
                        setShowCreateModal(false);
                        setClickedCoords(null);
                    }}
                    onCreated={() => {
                        setShowCreateModal(false);
                        setClickedCoords(null);
                        loadArticles();
                    }}
                />
            )}
        </div>
    );
}
