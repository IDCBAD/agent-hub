import {
  DatabaseIcon,
  FolderOpenIcon,
  MagnifyingGlassIcon,
} from "@phosphor-icons/react";
import { copy } from "../../shared/i18n/zh-CN";

const items = [
  {
    icon: DatabaseIcon,
    title: copy.settings.dataTitle,
    body: copy.settings.dataBody,
  },
  {
    icon: MagnifyingGlassIcon,
    title: copy.settings.scanTitle,
    body: copy.settings.scanBody,
  },
  {
    icon: FolderOpenIcon,
    title: copy.settings.openTitle,
    body: copy.settings.openBody,
  },
];

export function SettingsPage() {
  return (
    <section className="page">
      <header className="page-header">
        <div>
          <h1>{copy.settings.title}</h1>
          <p>{copy.settings.subtitle}</p>
        </div>
      </header>
      <div className="settings-list">
        {items.map((item) => {
          const Icon = item.icon;
          return (
            <article key={item.title}>
              <Icon size={21} />
              <div>
                <h2>{item.title}</h2>
                <p>{item.body}</p>
              </div>
            </article>
          );
        })}
      </div>
    </section>
  );
}
