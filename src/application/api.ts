import { invoke } from "@tauri-apps/api/core";
import type {
  Attachment,
  Bootstrap,
  BrandKit,
  Project,
  ProjectSummary,
  Provider,
  ProviderInput,
  VisualVersion,
} from "../domain/types";

export const api = {
  bootstrap: () => invoke<Bootstrap>("bootstrap"),
  project: (id: string) => invoke<Project | null>("get_project", { id }),
  createProject: (title: string, brandKitId: string, format: string) =>
    invoke<Project>("create_project", { title, brandKitId, format }),
  updateProject: (
    id: string,
    title: string,
    brandKitId: string,
    format: string,
  ) => invoke<Project>("update_project", { id, title, brandKitId, format }),
  deleteProject: (id: string) => invoke<void>("delete_project", { id }),
  saveBrandKit: (kit: BrandKit) => invoke<BrandKit>("save_brand_kit", { kit }),
  generateBrandKit: (name: string) =>
    invoke<BrandKit>("generate_brand_kit", { name }),
  deleteBrandKit: (id: string) => invoke<void>("delete_brand_kit", { id }),
  saveProvider: (input: ProviderInput) =>
    invoke<Provider>("save_provider", { input }),
  removeProvider: (id: string) => invoke<void>("remove_provider", { id }),
  activateProvider: (id: string) => invoke<void>("activate_provider", { id }),
  checkLocalProviders: () =>
    invoke<[string, boolean][]>("check_local_providers"),
  generate: (
    projectId: string,
    prompt: string,
    baseVersionId?: string,
    attachments: Attachment[] = [],
  ) =>
    invoke<VisualVersion>("generate_version", {
      input: {
        projectId,
        prompt,
        baseVersionId,
        attachments: attachments.map(({ previewUrl, ...file }) => file),
      },
    }),
  export: (versionId: string, extension: "svg" | "png", path: string) =>
    invoke<void>("export_version", { versionId, extension, path }),
  renderSvg: (versionId: string) =>
    invoke<string>("render_version_svg", { versionId }),
};
