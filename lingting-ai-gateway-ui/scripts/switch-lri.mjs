import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const FILES = {
  vite: path.join(ROOT, "vite.config.ts"),
  ts: path.join(ROOT, "tsconfig.json"),
  pkg: path.join(ROOT, "package.json"),
};

const LRI_DIR = path.join(ROOT, "lri");
const LRI_PACKAGE = "lingting-react-ui";

const TARGETS = {
  local: {
    vite: `resolve(import.meta.dirname, "lri")`,
    ts: "./lri",
  },
  package: {
    vite: `"${LRI_PACKAGE}"`,
    ts: `./node_modules/${LRI_PACKAGE}`,
  },
};

// 用法:
//   node scripts/switch-lri.mjs          # 自动切换
//   node scripts/switch-lri.mjs local    # 切换到本地 lri
//   node scripts/switch-lri.mjs package  # 切换到 lingting-react-ui
//   node scripts/switch-lri.mjs status   # 查看当前状态

const read = (file) => fs.readFileSync(file, "utf8");
const write = (file, content) => fs.writeFileSync(file, content);

function requireFile(file) {
  if (!fs.existsSync(file)) {
    throw new Error(`文件不存在: ${path.relative(ROOT, file)}`);
  }
}

function replaceLine(file, key, value) {
  const content = read(file);
  const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(`^(\\s*)${escaped}.*$`, "m");

  if (!re.test(content)) {
    throw new Error(`${path.basename(file)} 未找到: ${key}`);
  }

  write(file, content.replace(re, `$1${value}`));
}

function detectMode() {
  const content = read(FILES.vite);

  if (content.includes(`"@lri": ${TARGETS.package.vite}`)) {
    return "package";
  }

  if (content.includes(`"@lri": ${TARGETS.local.vite}`)) {
    return "local";
  }

  return "unknown";
}

function setViteConfig(mode) {
  replaceLine(FILES.vite, '"@lri":', `  "@lri": ${TARGETS[mode].vite},`);
}

function setTsConfig(mode) {
  const target = TARGETS[mode].ts;

  replaceLine(FILES.ts, '"@lri":', `      "@lri": ["${target}"],`);
  replaceLine(FILES.ts, '"@lri/*":', `      "@lri/*": ["${target}/*"]`);
}

function checkLocal() {
  if (!fs.existsSync(LRI_DIR) || !fs.statSync(LRI_DIR).isDirectory()) {
    throw new Error("本地 lri 不存在，无法切换到 local 模式。");
  }
}

function checkPackage() {
  const pkg = JSON.parse(read(FILES.pkg));
  const deps = {
    ...pkg.dependencies,
    ...pkg.devDependencies,
    ...pkg.peerDependencies,
  };

  if (!deps[LRI_PACKAGE]) {
    throw new Error(`package.json 未声明 ${LRI_PACKAGE}，无法切换到 package 模式。`);
  }
}

function removeLocalLri() {
  if (fs.existsSync(LRI_DIR)) {
    fs.rmSync(LRI_DIR, { recursive: true, force: true });
  }
}

function switchTo(mode) {
  if (mode === "local") {
    checkLocal();
  } else {
    checkPackage();
  }

  setViteConfig(mode);
  setTsConfig(mode);

  if (mode === "package") {
    removeLocalLri();
  }

  console.log(`已切换到 ${mode} 模式。`);
}

function status() {
  const mode = detectMode();
  const pkg = JSON.parse(read(FILES.pkg));
  const deps = {
    ...pkg.dependencies,
    ...pkg.devDependencies,
    ...pkg.peerDependencies,
  };

  console.log(`mode:             ${mode}`);
  console.log(`vite @lri:        ${TARGETS[mode]?.vite ?? "unknown"}`);
  console.log(`tsconfig @lri:    ${TARGETS[mode]?.ts ?? "unknown"}`);
  console.log(`local lri:        ${fs.existsSync(LRI_DIR) ? "exists" : "missing"}`);
  console.log(`package declared: ${deps[LRI_PACKAGE] ? "yes" : "no"}`);
}

try {
  Object.values(FILES).forEach(requireFile);

  const command = process.argv[2];

  if (!command) {
    const mode = detectMode();

    if (mode === "local") {
      switchTo("package");
    } else if (mode === "package") {
      switchTo("local");
    } else {
      throw new Error("无法识别当前 @lri 模式，请检查 vite.config.ts。");
    }
  } else if (command === "local" || command === "package") {
    switchTo(command);
  } else if (command === "status") {
    status();
  } else {
    console.error("用法: node scripts/switch-lri.mjs [local|package|status]");
    process.exit(1);
  }
} catch (error) {
  console.error(`错误: ${error.message}`);
  process.exit(1);
}
