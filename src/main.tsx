import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import {
  ArrowRight,
  ArrowUp,
  Download,
  FileText,
  MoreHorizontal,
  Paperclip,
  Plus,
  Settings,
  Sparkles,
  Trash2,
  X,
} from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { api } from "./application/api";
import { formats, palette, relativeTime } from "./domain/visual";
import type {
  Attachment,
  BrandKit,
  Bootstrap,
  Format,
  Project,
  ProjectSummary,
  Provider,
  ProviderInput,
  VisualVersion,
} from "./domain/types";
import "./styles.css";

type Screen = "home" | "brand" | "studio";
const blankBrand = (): BrandKit => ({
  id: "",
  name: "",
  sector: "",
  description: "",
  colors: [
    { name: "Fond", hex: "#F2EBDD", role: "Fond" },
    { name: "Texte", hex: "#2B1D16", role: "Texte" },
    { name: "Accent", hex: "#C8553D", role: "Accent" },
    { name: "Secondaire", hex: "#9BA88A", role: "Secondaire" },
  ],
  displayFont: "Archivo",
  bodyFont: "Onest",
  tone: [],
  dos: [],
  donts: [],
  createdAt: "",
});
function App() {
  const [data, setData] = useState<Bootstrap | null>(null);
  const [screen, setScreen] = useState<Screen>("home");
  const [project, setProject] = useState<Project | null>(null);
  const [settings, setSettings] = useState(false);
  const [brandEditor, setBrandEditor] = useState<BrandKit | null>(null);
  const [notice, setNotice] = useState("");
  const [initialGenerating, setInitialGenerating] = useState(false);
  const refresh = async () => setData(await api.bootstrap());
  useEffect(() => {
    void refresh().catch((e) => setNotice(String(e)));
  }, []);
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setSettings(true);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
  const open = async (id: string) => {
    const p = await api.project(id);
    if (p) {
      setProject(p);
      setScreen("studio");
    }
  };
  const create = async (
    brief: string,
    brandId: string,
    format: Format,
    attachments: Attachment[],
  ) => {
    if (!brief.trim()) return;
    const title = brief.trim().slice(0, 55);
    const p = await api.createProject(title, brandId, format);
    setInitialGenerating(true);
    setProject(p);
    setScreen("studio");
    try {
      const v = await api.generate(p.id, brief, undefined, attachments);
      setProject({ ...p, versions: [v] });
      await refresh();
    } catch (e) {
      setNotice(e instanceof Error ? e.message : String(e));
    } finally {
      setInitialGenerating(false);
    }
  };
  if (!data)
    return (
      <div className="app-frame">
        <div className="app">
          <div className="loader">Ouverture de l’atelier local…</div>
        </div>
      </div>
    );
  const activeProvider = data.providers.find((x) => x.enabled);
  const brandForProject =
    project && data.brandKits.find((x) => x.id === project.brandKitId);
  return (
    <div className="app-frame">
      <main className="app">
        <TopBar
          screen={screen}
          setScreen={setScreen}
          onSettings={() => setSettings(true)}
          hasProvider={Boolean(activeProvider)}
        />
        {screen === "home" && (
          <Home
            projects={data.projects}
            brands={data.brandKits}
            activeProvider={Boolean(activeProvider)}
            onCreate={create}
            onOpen={open}
            onDelete={async (id) => {
              await api.deleteProject(id);
              await refresh();
            }}
          />
        )}
        {screen === "brand" && (
          <BrandLibrary
            brands={data.brandKits}
            projects={data.projects}
            onEdit={setBrandEditor}
            onCreate={() => setBrandEditor(blankBrand())}
          />
        )}
        {screen === "studio" && project && brandForProject && (
          <Studio
            project={project}
            brand={brandForProject}
            brands={data.brandKits}
            activeProvider={activeProvider}
            initialGenerating={initialGenerating}
            onBack={() => setScreen("home")}
            onProject={async (p) => {
              setProject(p);
              await refresh();
            }}
            onSetNotice={setNotice}
          />
        )}
        {settings && (
          <SettingsModal
            providers={data.providers}
            onClose={() => setSettings(false)}
            onChange={refresh}
            onNotice={setNotice}
          />
        )}
        {brandEditor && (
          <BrandModal
            initialName={brandEditor.name}
            hasProvider={Boolean(activeProvider)}
            onClose={() => setBrandEditor(null)}
            onCreated={async () => {
              setBrandEditor(null);
              await refresh();
            }}
            onNotice={setNotice}
          />
        )}
        {notice && (
          <div className="alert" onClick={() => setNotice("")}>
            {notice}
          </div>
        )}
      </main>
    </div>
  );
}
function TopBar({
  screen,
  setScreen,
  onSettings,
  hasProvider,
}: {
  screen: Screen;
  setScreen: (x: Screen) => void;
  onSettings: () => void;
  hasProvider: boolean;
}) {
  return (
    <header className="topbar">
      <div className="brand">
        <i className="logo" />
        OpenStudio
      </div>
      <nav className="nav">
        {(
          [
            ["home", "Créations"],
            ["brand", "Directions artistiques"],
            ["studio", "Studio"],
          ] as [Screen, string][]
        ).map(([id, label]) => (
          <button
            key={id}
            className={screen === id ? "active" : ""}
            onClick={() => setScreen(id)}
          >
            {label}
          </button>
        ))}
      </nav>
      <div className="top-actions">
        <button className="shortcut" onClick={onSettings}>
          ⌘ K
        </button>
        <button
          className="avatar"
          title="Réglages des modèles IA"
          onClick={onSettings}
        >
          {hasProvider ? "AI" : "—"}
        </button>
      </div>
    </header>
  );
}
function Home({
  projects,
  brands,
  activeProvider,
  onCreate,
  onOpen,
  onDelete,
}: {
  projects: ProjectSummary[];
  brands: BrandKit[];
  activeProvider: boolean;
  onCreate: (b: string, id: string, f: Format, a: Attachment[]) => void;
  onOpen: (id: string) => void;
  onDelete: (id: string) => Promise<void>;
}) {
  const [brief, setBrief] = useState("");
  const [brandId, setBrandId] = useState(brands[0]?.id ?? "");
  const [format, setFormat] = useState<Format>("post");
  const [filter, setFilter] = useState("Toutes");
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  useEffect(() => {
    if (!brandId && brands[0]) setBrandId(brands[0].id);
  }, [brands, brandId]);
  const shown = projects.filter(
    (p) => filter === "Toutes" || formats[p.format].label === filter,
  );
  return (
    <section className="home">
      <div className="home-inner">
        <div className="hero">
          <h1>Que veux-tu créer aujourd’hui&nbsp;?</h1>
          <div className="composer">
            <textarea
              value={brief}
              onChange={(e) => setBrief(e.target.value)}
              placeholder="Une story pour annoncer l’ouverture du samedi matin…"
            />
            <AttachmentPicker
              attachments={attachments}
              onChange={setAttachments}
            />
            <div className="composer-row">
              <select
                className="select-chip"
                value={brandId}
                onChange={(e) => setBrandId(e.target.value)}
              >
                <option value="">Direction artistique</option>
                {brands.map((b) => (
                  <option value={b.id} key={b.id}>
                    {b.name}
                  </option>
                ))}
              </select>
              <select
                className="select-chip"
                value={format}
                onChange={(e) => setFormat(e.target.value as Format)}
              >
                {Object.entries(formats).map(([id, x]) => (
                  <option value={id} key={id}>
                    {x.label} · {x.ratio}
                  </option>
                ))}
              </select>
              <i className="spacer" />
              <button
                className="primary"
                disabled={!activeProvider || !brandId || !brief.trim()}
                onClick={() => onCreate(brief, brandId, format, attachments)}
              >
                Générer <ArrowRight size={14} />
              </button>
            </div>
            {!activeProvider && (
              <small className="muted">
                Connecte un modèle IA dans les réglages pour créer.
              </small>
            )}
            {!brands.length && (
              <small className="muted">
                Crée d’abord une direction artistique.
              </small>
            )}
          </div>
        </div>
        <div>
          <div className="section-title">
            <h2>Mes créations</h2>
            <div className="filters">
              {["Toutes", "Post", "Story", "Carré", "Paysage"].map((x) => (
                <button
                  className={"filter " + (filter === x ? "active" : "")}
                  onClick={() => setFilter(x)}
                  key={x}
                >
                  {x}
                </button>
              ))}
            </div>
          </div>
          <div className="visual-gallery" style={{ marginTop: 18 }}>
            {shown.map((p) => (
              <ProjectCard
                project={p}
                brand={brands.find((b) => b.id === p.brandKitId)}
                onOpen={onOpen}
                onDelete={onDelete}
                key={p.id}
              />
            ))}
          </div>
          {!shown.length && (
            <div className="empty" style={{ marginTop: 18 }}>
              <strong>Aucune création ici</strong>Les visuels générés par votre
              modèle IA resteront dans cet atelier local.
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
function AttachmentPicker({
  attachments,
  onChange,
}: {
  attachments: Attachment[];
  onChange: (items: Attachment[]) => void;
}) {
  const input = useRef<HTMLInputElement>(null);
  const add = async (files: FileList | File[]) => {
    const accepted = [...files]
      .filter(
        (f) =>
          f.type.startsWith("image/") ||
          f.type === "application/pdf" ||
          f.type.startsWith("text/"),
      )
      .slice(0, 8 - attachments.length);
    const next = await Promise.all(accepted.map(fileToAttachment));
    onChange([...attachments, ...next]);
  };
  const drop = (e: React.DragEvent) => {
    e.preventDefault();
    void add(e.dataTransfer.files);
  };
  return (
    <div
      className="attachment-zone"
      onDragOver={(e) => e.preventDefault()}
      onDrop={drop}
    >
      <input
        ref={input}
        type="file"
        hidden
        multiple
        accept="image/*,application/pdf,text/plain"
        onChange={(e) => {
          if (e.target.files) void add(e.target.files);
          e.target.value = "";
        }}
      />
      <button
        className="attach-button"
        onClick={() => input.current?.click()}
        type="button"
      >
        <Paperclip size={15} /> Ajouter une inspiration
      </button>
      <span className="muted">ou glisse tes images, PDF ou textes</span>
      {attachments.length > 0 && (
        <div className="attachment-list">
          {attachments.map((a, index) => (
            <div className="attachment-chip" key={`${a.name}-${index}`}>
              {a.previewUrl ? (
                <img src={a.previewUrl} alt="" />
              ) : (
                <FileText size={14} />
              )}
              <span>{a.name}</span>
              <button
                type="button"
                aria-label={`Retirer ${a.name}`}
                onClick={() =>
                  onChange(attachments.filter((_, i) => i !== index))
                }
              >
                <X size={13} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
function fileToAttachment(file: File): Promise<Attachment> {
  return new Promise((resolve, reject) => {
    if (file.size > 10 * 1024 * 1024) {
      reject(new Error(`${file.name} dépasse la limite de 10 Mo.`));
      return;
    }
    const reader = new FileReader();
    reader.onerror = () =>
      reject(new Error(`Impossible de lire ${file.name}.`));
    reader.onload = () => {
      const dataUrl = String(reader.result);
      resolve({
        name: file.name,
        mimeType: file.type || "application/octet-stream",
        dataBase64: dataUrl.split(",")[1],
        previewUrl: file.type.startsWith("image/") ? dataUrl : undefined,
      });
    };
    reader.readAsDataURL(file);
  });
}
function ProjectCard({
  project,
  brand,
  onOpen,
  onDelete,
}: {
  project: ProjectSummary;
  brand?: BrandKit;
  onOpen: (x: string) => void;
  onDelete: (x: string) => Promise<void>;
}) {
  const [menu, setMenu] = useState(false);
  const v = project.latestVersion;
  const p = brand && v ? palette(brand, v.spec.scheme) : null;
  const f = formats[project.format];
  return (
    <article className="gallery-item">
      <button className="gallery-visual" onClick={() => onOpen(project.id)}>
        {v && brand && p ? (
          <div
            className="visual"
            style={{
              width: Math.min(220, f.width * 0.5),
              height: Math.min(250, f.height * 0.46),
              background: p.bg,
              color: p.fg,
              padding: 12,
              fontFamily: brand.displayFont,
            }}
          >
            <i className="visual-circle" style={{ background: p.shape }} />
            <div className="visual-top" style={{ fontSize: 6 }}>
              {brand.name}
            </div>
            <div className={"visual-main " + v.spec.alignment}>
              {v.spec.kicker && (
                <small className="visual-kicker">{v.spec.kicker}</small>
              )}
              <h2 style={{ fontSize: Math.min(23, v.spec.titleSize * 0.32) }}>
                {v.spec.headline}
              </h2>
              {v.spec.details?.slice(0, 2).map((d) => (
                <small className="visual-detail" key={d}>
                  {d}
                </small>
              ))}
            </div>
          </div>
        ) : (
          <span className="muted">Aucune version</span>
        )}
      </button>
      <button
        className="gallery-options"
        aria-label={`Options pour ${project.title}`}
        onClick={() => setMenu(!menu)}
      >
        <MoreHorizontal size={18} />
      </button>
      {menu && (
        <div className="gallery-menu">
          <button
            onClick={async () => {
              if (window.confirm(`Supprimer « ${project.title} » ?`))
                await onDelete(project.id);
            }}
          >
            <Trash2 size={14} /> Supprimer
          </button>
        </div>
      )}
      <div className="gallery-caption">
        <span>{project.title}</span>
        {v && <span className="mono">V{v.sequence + 1}</span>}
      </div>
    </article>
  );
}
function BrandLibrary({
  brands,
  projects,
  onEdit,
  onCreate,
}: {
  brands: BrandKit[];
  projects: ProjectSummary[];
  onEdit: (b: BrandKit) => void;
  onCreate: () => void;
}) {
  const [selected, setSelected] = useState(brands[0]?.id);
  useEffect(() => {
    if (!selected && brands[0]) setSelected(brands[0].id);
  }, [brands, selected]);
  const b = brands.find((x) => x.id === selected);
  return (
    <section className="da-page">
      <aside className="da-list">
        <div className="list-heading">Directions artistiques</div>
        {brands.map((x) => (
          <button
            className={"da-row " + (x.id === selected ? "active" : "")}
            onClick={() => setSelected(x.id)}
            key={x.id}
          >
            <div className="da-row-head">
              <span>
                <span className="da-name">{x.name}</span>
                <br />
                <span className="muted">{x.sector}</span>
              </span>
              <span className="mono">
                {projects.filter((p) => p.brandKitId === x.id).length} créas
              </span>
            </div>
            <div className="palette-strip">
              {x.colors.map((c) => (
                <i style={{ background: c.hex }} key={c.hex} />
              ))}
            </div>
          </button>
        ))}
        <button className="dashed" onClick={onCreate}>
          + Nouvelle DA
        </button>
      </aside>
      <div className="da-detail">
        {b ? (
          <BrandDetail brand={b} onEdit={() => onEdit(b)} />
        ) : (
          <div className="empty">
            <strong>Votre première direction artistique</strong>Ajoutez
            couleurs, polices et règles de ton pour guider vos créations.
          </div>
        )}
      </div>
    </section>
  );
}
function BrandDetail({
  brand,
  onEdit,
}: {
  brand: BrandKit;
  onEdit: () => void;
}) {
  return (
    <div className="da-content">
      <div className="da-header">
        <div>
          <div className="eyebrow">{brand.sector}</div>
          <h1>{brand.name}</h1>
          <div className="muted">{brand.description}</div>
        </div>
        <button className="primary" onClick={onEdit}>
          Nouvelle proposition
        </button>
      </div>
      <div className="grid-2">
        <section className="panel">
          <div className="panel-head">
            Logo <span className="muted">identité typographique</span>
          </div>
          <div
            className="logo-preview"
            style={{
              background: brand.colors[0]?.hex,
              color: brand.colors[1]?.hex,
              fontFamily: brand.displayFont,
            }}
          >
            {brand.name}
          </div>
        </section>
        <section className="panel">
          <div className="panel-head">
            Charte couleurs{" "}
            <span className="muted">{brand.colors.length} couleurs</span>
          </div>
          <div className="color-grid">
            {brand.colors.map((c) => (
              <div className="color-card" key={c.hex}>
                <div className="swatch" style={{ background: c.hex }} />
                <div className="color-meta">
                  {c.name}
                  <small>
                    <span>{c.hex}</span>
                    <span>{c.role}</span>
                  </small>
                </div>
              </div>
            ))}
          </div>
        </section>
      </div>
      <section className="panel">
        <div className="panel-head">Typographie</div>
        <div className="grid-2">
          <div className="font-row">
            <div className="font-aa" style={{ fontFamily: brand.displayFont }}>
              Aa
            </div>
            <div>
              <div className="eyebrow">Titres</div>
              <strong>{brand.displayFont}</strong>
              <p className="muted">Accroches, titres, logo</p>
            </div>
          </div>
          <div className="font-row">
            <div
              className="font-aa"
              style={{ fontFamily: brand.bodyFont, fontWeight: 400 }}
            >
              Aa
            </div>
            <div>
              <div className="eyebrow">Texte</div>
              <strong>{brand.bodyFont}</strong>
              <p className="muted">Sous-titres, informations</p>
            </div>
          </div>
        </div>
      </section>
      <section className="panel">
        <div className="panel-head">Brand guide · Ton de voix</div>
        <div className="tone">
          {brand.tone.map((x) => (
            <span key={x}>{x}</span>
          ))}
        </div>
        <div className="rules" style={{ marginTop: 16 }}>
          <div>
            <h4 style={{ color: "#398160" }}>À FAIRE</h4>
            {brand.dos.map((x) => (
              <p key={x}>{x}</p>
            ))}
          </div>
          <div>
            <h4 style={{ color: "#b84b38" }}>À ÉVITER</h4>
            {brand.donts.map((x) => (
              <p key={x}>{x}</p>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
}
function Studio({
  project,
  brand,
  brands,
  activeProvider,
  initialGenerating,
  onBack,
  onProject,
  onSetNotice,
}: {
  project: Project;
  brand: BrandKit;
  brands: BrandKit[];
  activeProvider?: Provider;
  initialGenerating: boolean;
  onBack: () => void;
  onProject: (p: Project) => void;
  onSetNotice: (x: string) => void;
}) {
  const [current, setCurrent] = useState(project.versions.at(-1)?.id);
  const [draft, setDraft] = useState("");
  const [attachments, setAttachments] = useState<Attachment[]>([]);
  const [busy, setBusy] = useState(false);
  const [tab, setTab] = useState<"da" | "format">("da");
  const [previewSvg, setPreviewSvg] = useState("");
  const version =
    project.versions.find((v) => v.id === current) ?? project.versions.at(-1);
  useEffect(() => {
    let active = true;
    if (!version) {
      setPreviewSvg("");
      return;
    }
    void api
      .renderSvg(version.id)
      .then((svg) => active && setPreviewSvg(svg))
      .catch((error) =>
        onSetNotice(error instanceof Error ? error.message : String(error)),
      );
    return () => {
      active = false;
    };
  }, [version?.id]);
  useEffect(() => setCurrent(project.versions.at(-1)?.id), [project.id]);
  const send = async (text = draft) => {
    if ((!text.trim() && !attachments.length) || busy) return;
    if (!activeProvider) {
      onSetNotice("Connecte un modèle IA actif dans les réglages.");
      return;
    }
    const files = attachments;
    document.querySelector(".canvas")?.classList.add("is-generating");
    setBusy(true);
    setDraft("");
    setAttachments([]);
    try {
      const v = await api.generate(
        project.id,
        text || "Utilise les fichiers joints comme référence visuelle.",
        current,
        files,
      );
      onProject({ ...project, versions: [...project.versions, v] });
      setCurrent(v.id);
      requestAnimationFrame(() => {
        const canvas = document.querySelector(".canvas");
        canvas?.classList.remove("is-generating");
        canvas?.classList.add("is-arriving");
        window.setTimeout(() => canvas?.classList.remove("is-arriving"), 520);
      });
    } catch (e) {
      document.querySelector(".canvas")?.classList.remove("is-generating");
      setAttachments(files);
      onSetNotice(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };
  const changeFormat = async (f: Format) => {
    try {
      onProject(
        await api.updateProject(
          project.id,
          project.title,
          project.brandKitId,
          f,
        ),
      );
    } catch (e) {
      onSetNotice(String(e));
    }
  };
  const exportCurrent = async () => {
    if (!version) return;
    const extension = await new Promise<"svg" | "png" | undefined>(
      (resolve) => {
        const answer = window.confirm(
          "Exporter en PNG ?\nOK = PNG · Annuler = SVG",
        );
        resolve(answer ? "png" : "svg");
      },
    );
    if (!extension) return;
    const path = await save({
      defaultPath: `${project.title.replaceAll("/", "-")}.${extension}`,
      filters: [{ name: extension.toUpperCase(), extensions: [extension] }],
    });
    if (path) {
      try {
        await api.export(version.id, extension, path);
        onSetNotice("Export enregistré.");
      } catch (e) {
        onSetNotice(String(e));
      }
    }
  };
  if (!version)
    return (
      <section className="studio">
        <div className="chat">
          <div className="chat-head">
            <div>
              <div className="chat-title">{project.title}</div>
              <div className="muted">
                {initialGenerating ? "Analyse du brief · " : "0 version · "}
                {brand.name}
              </div>
            </div>
            <button className="pill" onClick={onBack}>
              Retour
            </button>
          </div>
          <div className="chat-log">
            <div className="ai-message">
              <i className="mini-logo" />
              <span>
                {initialGenerating
                  ? "L’IA analyse le brief, la DA et les inspirations jointes."
                  : "Décris le visuel à réaliser. La direction artistique sera appliquée à chaque version."}
              </span>
            </div>
          </div>
          <div className="chat-compose">
            <AttachmentPicker
              attachments={attachments}
              onChange={setAttachments}
            />
            <div className="chat-input">
              <input
                autoFocus
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && send()}
                placeholder="Décris le visuel ou joins une inspiration…"
                disabled={initialGenerating}
              />
              <button
                className="send"
                disabled={initialGenerating}
                onClick={() => send()}
              >
                <ArrowUp size={15} />
              </button>
            </div>
          </div>
        </div>
        <div className={"canvas " + (initialGenerating ? "is-generating" : "")}>
          <div className={initialGenerating ? "initial-generation" : "empty"}>
            {initialGenerating ? (
              <>
                <i className="generation-mark">
                  <Sparkles size={18} />
                </i>
                <strong>Composition en cours</strong>
                <span>
                  L’IA construit la première version à partir de ton brief.
                </span>
                <div className="generation-steps">
                  <span>Brief</span>
                  <span>DA</span>
                  <span>Composition</span>
                </div>
              </>
            ) : (
              <>
                <strong>Prêt à composer</strong>Envoie un brief ou une
                inspiration à l’IA pour générer la première version.
              </>
            )}
          </div>
        </div>
        <Inspector
          brand={brand}
          brands={brands}
          format={project.format}
          tab={tab}
          setTab={setTab}
          changeFormat={changeFormat}
        />
      </section>
    );
  const f = formats[project.format];
  const scale = Math.min(1, 520 / f.height, 540 / f.width);
  return (
    <section className="studio">
      <div className="chat">
        <div className="chat-head">
          <div>
            <div className="chat-title">{project.title}</div>
            <div className="muted">
              {project.versions.length} versions · {brand.name}
            </div>
          </div>
          <button className="pill" onClick={onBack}>
            Retour
          </button>
        </div>
        <div className="chat-log">
          <div className="message">{project.versions[0]?.prompt}</div>
          {project.versions.map((v) => (
            <div className="ai-message" key={v.id}>
              <i className="mini-logo" />
              <div>
                <span>{v.label}</span>
                <button
                  className={
                    "version-chip " + (version.id === v.id ? "active" : "")
                  }
                  onClick={() => setCurrent(v.id)}
                >
                  <i
                    className="mini-thumb"
                    style={{ background: palette(brand, v.spec.scheme).bg }}
                  />
                  <span>
                    <b>Version {v.sequence + 1}</b>
                    <br />
                    <small className="muted">{v.label}</small>
                  </span>
                </button>
              </div>
            </div>
          ))}
          {busy && (
            <div className="ai-message">
              <i className="mini-logo" />
              <span>Le modèle prépare la prochaine version…</span>
            </div>
          )}
        </div>
        <div className="chat-compose">
          <div className="tone" style={{ marginBottom: 10 }}>
            {[
              "Plus contrasté",
              "Agrandis le titre",
              "Change l’accroche",
              "Centre le texte",
            ].map((x) => (
              <button className="filter" onClick={() => send(x)} key={x}>
                {x}
              </button>
            ))}
          </div>
          <AttachmentPicker
            attachments={attachments}
            onChange={setAttachments}
          />
          <div className="chat-input">
            <input
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && send()}
              placeholder="Donne un retour ou joins un screen…"
              disabled={busy}
            />
            <button className="send" disabled={busy} onClick={() => send()}>
              <ArrowUp size={15} />
            </button>
          </div>
        </div>
      </div>
      <div className="canvas">
        <div className="canvas-top">
          <div className="canvas-label">
            <span className="mono">V{version.sequence + 1}</span> ·{" "}
            {version.label}
          </div>
          <button className="primary" onClick={exportCurrent}>
            Exporter <Download size={14} />
          </button>
        </div>
        <div
          className="visual svg-preview"
          style={{ width: f.width * scale, height: f.height * scale }}
        >
          {previewSvg ? (
            <img
              src={`data:image/svg+xml;charset=utf-8,${encodeURIComponent(previewSvg)}`}
              alt={`Aperçu de ${version.label}`}
            />
          ) : (
            <span className="muted">Mise à jour de l’aperçu…</span>
          )}
        </div>
        <div className="versions">
          <span className="muted">Versions</span>
          {project.versions.map((v) => (
            <button
              className={v.id === version.id ? "active" : ""}
              onClick={() => setCurrent(v.id)}
              key={v.id}
            >
              V{v.sequence + 1}
            </button>
          ))}
        </div>
      </div>
      <Inspector
        brand={brand}
        brands={brands}
        format={project.format}
        tab={tab}
        setTab={setTab}
        changeFormat={changeFormat}
      />
    </section>
  );
}
function Inspector({
  brand,
  brands,
  format,
  tab,
  setTab,
  changeFormat,
}: {
  brand: BrandKit;
  brands: BrandKit[];
  format: Format;
  tab: "da" | "format";
  setTab: (x: "da" | "format") => void;
  changeFormat: (x: Format) => void;
}) {
  return (
    <aside className="right">
      <div className="right-tabs">
        <button
          className={"tab " + (tab === "da" ? "active" : "")}
          onClick={() => setTab("da")}
        >
          Direction artistique
        </button>
        <button
          className={"tab " + (tab === "format" ? "active" : "")}
          onClick={() => setTab("format")}
        >
          Format
        </button>
      </div>
      <div className="inspector">
        {tab === "da" ? (
          <>
            <h3>DA appliquée</h3>
            {brands.map((b) => (
              <div
                className={"link-row " + (b.id === brand.id ? "active" : "")}
                key={b.id}
              >
                <i
                  className="initial"
                  style={{
                    background: b.colors[0]?.hex,
                    color: b.colors[1]?.hex,
                    fontFamily: b.displayFont,
                  }}
                >
                  {b.name[0]}
                </i>
                <span>
                  <b className="da-name">{b.name}</b>
                  <br />
                  <small className="muted">
                    {b.id === brand.id
                      ? "Appliquée à la création"
                      : "Disponible dans la bibliothèque"}
                  </small>
                </span>
              </div>
            ))}
            <div className="inspector-card">
              <small className="muted">Couleurs</small>
              <div className="small-swatches">
                {brand.colors.map((c) => (
                  <i key={c.hex} style={{ background: c.hex }} title={c.hex} />
                ))}
              </div>
            </div>
            <div className="inspector-card">
              <small className="muted">Ton de voix</small>
              <div className="tone" style={{ marginTop: 8 }}>
                {brand.tone.map((t) => (
                  <span key={t}>{t}</span>
                ))}
              </div>
            </div>
          </>
        ) : (
          <>
            <h3>Format</h3>
            {(
              Object.entries(formats) as [Format, (typeof formats)[Format]][]
            ).map(([id, x]) => {
              const side = 20 / Math.max(x.width, x.height);
              return (
                <button
                  className={"format-row " + (format === id ? "active" : "")}
                  onClick={() => changeFormat(id)}
                  key={id}
                >
                  <i className="format-icon">
                    <i
                      style={{ width: x.width * side, height: x.height * side }}
                    />
                  </i>
                  <span className="spacer">
                    <b className="da-name">{x.label}</b>
                    <br />
                    <small className="muted">{x.detail}</small>
                  </span>
                  <span className="mono">{x.ratio}</span>
                </button>
              );
            })}
            <p className="muted">
              Le changement de format réadapte le canevas sans recréer une
              version.
            </p>
          </>
        )}
      </div>
    </aside>
  );
}
function SettingsModal({
  providers,
  onClose,
  onChange,
  onNotice,
}: {
  providers: Provider[];
  onClose: () => void;
  onChange: () => Promise<void>;
  onNotice: (x: string) => void;
}) {
  const [form, setForm] = useState<ProviderInput>({
    kind: "codex_cli",
    name: "Codex local",
    model: "",
    enabled: false,
  });
  const [detected, setDetected] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  useEffect(() => {
    void api
      .checkLocalProviders()
      .then((xs) => setDetected(xs.filter((x) => x[1]).map((x) => x[0])))
      .catch(() => undefined);
  }, []);
  const saveProvider = async () => {
    if (!form.name.trim()) {
      onNotice("Donne un nom à cette connexion.");
      return;
    }
    if (
      ["open_ai_compatible", "anthropic"].includes(form.kind) &&
      !form.apiKey &&
      !form.id
    ) {
      onNotice("Une clé API est requise pour ce fournisseur.");
      return;
    }
    setSaving(true);
    try {
      await api.saveProvider(form);
      await onChange();
      setForm({
        kind: "codex_cli",
        name: "Codex local",
        model: "",
        enabled: false,
      });
    } catch (e) {
      onNotice(String(e));
    } finally {
      setSaving(false);
    }
  };
  return (
    <div className="modal-backdrop">
      <section className="modal">
        <button
          style={{ float: "right", background: "transparent" }}
          onClick={onClose}
        >
          <X size={18} />
        </button>
        <h2>Modèles IA</h2>
        <p>
          OpenStudio n’envoie rien à un serveur applicatif. Les clés BYOK
          restent dans le trousseau de votre système. Un modèle actif est
          nécessaire pour créer.
        </p>
        <div className="form-grid">
          <label className="field">
            Type
            <select
              value={form.kind}
              onChange={(e) => {
                const kind = e.target.value as ProviderInput["kind"];
                setForm({
                  ...form,
                  kind,
                  name:
                    kind === "codex_cli"
                      ? "Codex local"
                      : kind === "claude_cli"
                        ? "Claude local"
                        : "Mon fournisseur",
                  model: kind === "anthropic" ? "claude-sonnet-4-5" : "gpt-4.1",
                });
              }}
            >
              <option value="codex_cli">
                Codex CLI local{" "}
                {detected.includes("codex_cli") ? "· détecté" : ""}
              </option>
              <option value="claude_cli">
                Claude CLI local{" "}
                {detected.includes("claude_cli") ? "· détecté" : ""}
              </option>
                <option value="open_ai_compatible">
                  API compatible OpenAI · texte ou image
                </option>
              <option value="anthropic">API Anthropic</option>
            </select>
          </label>
          <label className="field">
            Nom
            <input
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
          </label>
          {!["codex_cli", "claude_cli"].includes(form.kind) && (
            <>
              <label className="field">
                Modèle
                <input
                  placeholder="gpt-4.1 ou gpt-image-2.5-flare"
                  value={form.model}
                  onChange={(e) => setForm({ ...form, model: e.target.value })}
                />
              </label>
              <label className="field">
                Clé API BYOK
                <input
                  type="password"
                  placeholder="Stockée dans le trousseau système"
                  value={form.apiKey ?? ""}
                  onChange={(e) => setForm({ ...form, apiKey: e.target.value })}
                />
              </label>
              <label className="field full">
                Endpoint{" "}
                {form.kind === "open_ai_compatible" ? "(optionnel)" : ""}
                <input
                  placeholder={
                    form.kind === "anthropic"
                      ? "https://api.anthropic.com/v1/messages"
                      : "https://api.openai.com/v1"
                  }
                  value={form.endpoint ?? ""}
                  onChange={(e) =>
                    setForm({ ...form, endpoint: e.target.value || undefined })
                  }
                />
              </label>
            </>
          )}
        </div>
        <button className="primary" disabled={saving} onClick={saveProvider}>
          <Plus size={14} /> Ajouter la connexion
        </button>
        <div style={{ marginTop: 20 }}>
          {providers.length ? (
            <>
              {providers.map((p) => (
                <div className="link-row active" key={p.id}>
                  <i className="initial" style={{ background: "#ececea" }}>
                    <Sparkles size={14} />
                  </i>
                  <span className="spacer">
                    <b className="da-name">{p.name}</b>
                    <br />
                    <small className="muted">
                      {p.kind.replaceAll("_", " ")} {p.model && `· ${p.model}`}{" "}
                      {p.hasSecret ? "· clé sécurisée" : ""}
                    </small>
                  </span>
                  {p.enabled ? (
                    <span className="mono" style={{ color: "#398160" }}>
                      actif
                    </span>
                  ) : (
                    <button
                      className="secondary"
                      onClick={async () => {
                        await api.activateProvider(p.id);
                        await onChange();
                      }}
                    >
                      Activer
                    </button>
                  )}
                  <button
                    className="secondary danger"
                    onClick={async () => {
                      await api.removeProvider(p.id);
                      await onChange();
                    }}
                  >
                    Retirer
                  </button>
                </div>
              ))}
            </>
          ) : (
            <div className="provider-empty">
              Aucun fournisseur configuré. Connectez le Codex ou Claude déjà
              installé, ou ajoutez votre endpoint et votre clé BYOK.
            </div>
          )}
        </div>
        <div className="modal-actions">
          <button className="secondary" onClick={onClose}>
            Fermer
          </button>
        </div>
      </section>
    </div>
  );
}
function BrandModal({
  initialName,
  hasProvider,
  onClose,
  onCreated,
  onNotice,
}: {
  initialName: string;
  hasProvider: boolean;
  onClose: () => void;
  onCreated: () => Promise<void>;
  onNotice: (x: string) => void;
}) {
  const [name, setName] = useState(initialName);
  const [creating, setCreating] = useState(false);
  const submit = async () => {
    if (!name.trim() || creating) return;
    if (!hasProvider) {
      onNotice(
        "Connecte un modèle IA actif pour créer une direction artistique.",
      );
      return;
    }
    setCreating(true);
    try {
      await api.generateBrandKit(name.trim());
      await onCreated();
    } catch (e) {
      onNotice(e instanceof Error ? e.message : String(e));
    } finally {
      setCreating(false);
    }
  };
  return (
    <div className="modal-backdrop">
      <section className="modal brand-starter">
        <button
          style={{ float: "right", background: "transparent" }}
          onClick={onClose}
          aria-label="Fermer"
        >
          <X size={18} />
        </button>
        <h2>Nom de la marque</h2>
        <p>
          OpenStudio compose la première direction artistique à partir de ce
          nom. Vous l’affinerez ensuite naturellement, à travers vos créations.
        </p>
        <div className="starter-input">
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && submit()}
            placeholder="ex. Maison Ourse"
          />
          <button
            className="send"
            disabled={creating || !name.trim()}
            onClick={submit}
            aria-label="Créer la direction artistique"
          >
            {creating ? (
              <span className="dot-pulse" />
            ) : (
              <ArrowRight size={16} />
            )}
          </button>
        </div>
        <div className="starter-note">
          <Sparkles size={14} /> Une première proposition est générée localement
          via votre modèle IA actif.
        </div>
      </section>
    </div>
  );
}

createRoot(document.getElementById("root")!).render(<App />);
