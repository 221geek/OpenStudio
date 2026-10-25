use crate::domain::models::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use uuid::Uuid;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        connection.execute_batch("PRAGMA foreign_keys = ON;
          CREATE TABLE IF NOT EXISTS brand_kits (id TEXT PRIMARY KEY, payload TEXT NOT NULL, created_at TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS projects (id TEXT PRIMARY KEY, title TEXT NOT NULL, brand_kit_id TEXT NOT NULL, format TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS versions (id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE, sequence INTEGER NOT NULL, label TEXT NOT NULL, prompt TEXT NOT NULL, spec TEXT NOT NULL, created_at TEXT NOT NULL, UNIQUE(project_id, sequence));
          CREATE TABLE IF NOT EXISTS providers (id TEXT PRIMARY KEY, kind TEXT NOT NULL, name TEXT NOT NULL, model TEXT NOT NULL, endpoint TEXT, enabled INTEGER NOT NULL, has_secret INTEGER NOT NULL, created_at TEXT NOT NULL);")
          .map_err(|e| e.to_string())?;
        Ok(Self { connection })
    }
    pub fn list_brand_kits(&self) -> Result<Vec<BrandKit>, String> {
        let mut s = self
            .connection
            .prepare("SELECT payload FROM brand_kits ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        let items = s
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .map(|r| {
                serde_json::from_str(&r.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
            })
            .collect();
        items
    }
    pub fn save_brand_kit(&self, kit: BrandKit) -> Result<BrandKit, String> {
        let payload = serde_json::to_string(&kit).map_err(|e| e.to_string())?;
        self.connection.execute("INSERT INTO brand_kits(id,payload,created_at) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",params![kit.id,payload,kit.created_at]).map_err(|e|e.to_string())?;
        Ok(kit)
    }
    pub fn delete_brand_kit(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM brand_kits WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn create_project(
        &self,
        title: String,
        brand_kit_id: String,
        format: String,
    ) -> Result<Project, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.connection
            .execute(
                "INSERT INTO projects VALUES(?1,?2,?3,?4,?5,?6)",
                params![id, title, brand_kit_id, format, now, now],
            )
            .map_err(|e| e.to_string())?;
        self.get_project(&id)?
            .ok_or("Project was not created".into())
    }
    pub fn get_project(&self, id: &str) -> Result<Option<Project>, String> {
        let project=self.connection.query_row("SELECT id,title,brand_kit_id,format,created_at,updated_at FROM projects WHERE id=?1",[id],|r|Ok(Project{id:r.get(0)?,title:r.get(1)?,brand_kit_id:r.get(2)?,format:r.get(3)?,created_at:r.get(4)?,updated_at:r.get(5)?,versions:vec![]})).optional().map_err(|e|e.to_string())?;
        project
            .map(|mut p| {
                p.versions = self.versions_for(&p.id)?;
                Ok(p)
            })
            .transpose()
    }
    pub fn list_projects(&self) -> Result<Vec<ProjectSummary>, String> {
        let mut s=self.connection.prepare("SELECT id,title,brand_kit_id,format,created_at,updated_at FROM projects ORDER BY updated_at DESC").map_err(|e|e.to_string())?;
        let items = s.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?))).map_err(|e|e.to_string())?.map(|row|{let (id,title,brand_kit_id,format,created_at,updated_at)=row.map_err(|e|e.to_string())?;let latest=self.connection.query_row("SELECT id,project_id,sequence,label,prompt,spec,created_at FROM versions WHERE project_id=?1 ORDER BY sequence DESC LIMIT 1",[&id],decode_version).optional().map_err(|e|e.to_string())?;Ok(ProjectSummary{id,title,brand_kit_id,format,created_at,updated_at,latest_version:latest})}).collect();
        items
    }
    pub fn update_project(
        &self,
        id: &str,
        title: String,
        brand_kit_id: String,
        format: String,
    ) -> Result<Project, String> {
        let now = Utc::now().to_rfc3339();
        self.connection
            .execute(
                "UPDATE projects SET title=?2,brand_kit_id=?3,format=?4,updated_at=?5 WHERE id=?1",
                params![id, title, brand_kit_id, format, now],
            )
            .map_err(|e| e.to_string())?;
        self.get_project(id)?.ok_or("Project not found".into())
    }
    pub fn delete_project(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM projects WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn append_version(
        &self,
        project_id: &str,
        label: String,
        prompt: String,
        spec: VisualSpec,
    ) -> Result<VisualVersion, String> {
        let sequence: i64 = self
            .connection
            .query_row(
                "SELECT COALESCE(MAX(sequence),-1)+1 FROM versions WHERE project_id=?1",
                [project_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        let item = VisualVersion {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.into(),
            sequence,
            label,
            prompt,
            spec,
            created_at: Utc::now().to_rfc3339(),
        };
        self.connection
            .execute(
                "INSERT INTO versions VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    item.id,
                    item.project_id,
                    item.sequence,
                    item.label,
                    item.prompt,
                    serde_json::to_string(&item.spec).map_err(|e| e.to_string())?,
                    item.created_at
                ],
            )
            .map_err(|e| e.to_string())?;
        self.connection
            .execute(
                "UPDATE projects SET updated_at=?2 WHERE id=?1",
                params![project_id, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        Ok(item)
    }
    fn versions_for(&self, id: &str) -> Result<Vec<VisualVersion>, String> {
        let mut s=self.connection.prepare("SELECT id,project_id,sequence,label,prompt,spec,created_at FROM versions WHERE project_id=?1 ORDER BY sequence ASC").map_err(|e|e.to_string())?;
        let items = s
            .query_map([id], decode_version)
            .map_err(|e| e.to_string())?
            .map(|x| x.map_err(|e| e.to_string()))
            .collect();
        items
    }
    pub fn list_providers(&self) -> Result<Vec<Provider>, String> {
        let mut s=self.connection.prepare("SELECT id,kind,name,model,endpoint,enabled,has_secret,created_at FROM providers ORDER BY created_at").map_err(|e|e.to_string())?;
        let items = s
            .query_map([], decode_provider)
            .map_err(|e| e.to_string())?
            .map(|x| x.map_err(|e| e.to_string()))
            .collect();
        items
    }
    pub fn save_provider(&self, p: Provider) -> Result<Provider, String> {
        self.connection.execute("INSERT INTO providers VALUES(?1,?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(id) DO UPDATE SET kind=excluded.kind,name=excluded.name,model=excluded.model,endpoint=excluded.endpoint,enabled=excluded.enabled,has_secret=excluded.has_secret",params![p.id,serde_json::to_string(&p.kind).map_err(|e|e.to_string())?.trim_matches('"'),p.name,p.model,p.endpoint,p.enabled as i32,p.has_secret as i32,p.created_at]).map_err(|e|e.to_string())?;
        Ok(p)
    }
    pub fn remove_provider(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM providers WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn enable_provider(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("UPDATE providers SET enabled=1 WHERE id=?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
fn decode_version(r: &rusqlite::Row) -> rusqlite::Result<VisualVersion> {
    let spec: String = r.get(5)?;
    Ok(VisualVersion {
        id: r.get(0)?,
        project_id: r.get(1)?,
        sequence: r.get(2)?,
        label: r.get(3)?,
        prompt: r.get(4)?,
        spec: serde_json::from_str(&spec).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
        })?,
        created_at: r.get(6)?,
    })
}
fn decode_provider(r: &rusqlite::Row) -> rusqlite::Result<Provider> {
    let kind: String = r.get(1)?;
    Ok(Provider {
        id: r.get(0)?,
        kind: serde_json::from_str(&format!("\"{kind}\"")).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e))
        })?,
        name: r.get(2)?,
        model: r.get(3)?,
        endpoint: r.get(4)?,
        enabled: r.get::<_, i32>(5)? != 0,
        has_secret: r.get::<_, i32>(6)? != 0,
        created_at: r.get(7)?,
    })
}
