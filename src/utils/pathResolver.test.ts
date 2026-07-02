import { describe, it, expect } from "vitest";
import {
  isAbsolutePath,
  toRelative,
  toAbsolute,
  normalizePath,
} from "./pathResolver";

describe("pathResolver", () => {
  describe("normalizePath", () => {
    it("converts backslashes to forward slashes", () => {
      expect(normalizePath("C:\\Users\\Admin\\project.glucose")).toBe(
        "C:/Users/Admin/project.glucose",
      );
    });

    it("strips file:// and file:/// prefix", () => {
      expect(normalizePath("file:///C:/path/file.png")).toBe("C:/path/file.png");
      expect(normalizePath("file://path/file.png")).toBe("path/file.png");
    });
  });

  describe("isAbsolutePath", () => {
    it("detects Windows absolute paths", () => {
      expect(isAbsolutePath("C:/Users/Admin")).toBe(true);
      expect(isAbsolutePath("d:\\assets")).toBe(true);
      expect(isAbsolutePath("file:///C:/path/file.png")).toBe(true);
    });

    it("detects Unix absolute paths", () => {
      expect(isAbsolutePath("/usr/bin/node")).toBe(true);
    });

    it("rejects relative paths", () => {
      expect(isAbsolutePath("assets/img.png")).toBe(false);
      expect(isAbsolutePath("../img.png")).toBe(false);
    });

    it("rejects web and custom asset protocols", () => {
      expect(isAbsolutePath("http://example.com/img.png")).toBe(false);
      expect(isAbsolutePath("https://example.com/img.png")).toBe(false);
      expect(isAbsolutePath("asset:123hash.png")).toBe(false);
      expect(isAbsolutePath("data:image/png;base64,123")).toBe(false);
    });
  });

  describe("toRelative", () => {
    it("computes relative path on the same Windows drive", () => {
      const project = "C:/Users/Admin/Documents/project.glucose";
      const file = "C:/Users/Admin/Documents/textures/image.png";
      expect(toRelative(file, project)).toBe("textures/image.png");

      const file2 = "C:/Users/Admin/image.png";
      expect(toRelative(file2, project)).toBe("../image.png");
    });

    it("keeps absolute path on different Windows drives", () => {
      const project = "C:/Users/Admin/project.glucose";
      const file = "D:/textures/image.png";
      expect(toRelative(file, project)).toBe("D:/textures/image.png");
    });

    it("handles file:// schema correctly", () => {
      const project = "C:/Users/Admin/project.glucose";
      const file = "file:///C:/Users/Admin/textures/image.png";
      expect(toRelative(file, project)).toBe("file://textures/image.png");
    });

    it("leaves relative and web paths unchanged", () => {
      expect(toRelative("textures/image.png")).toBe("textures/image.png");
      expect(toRelative("https://site.com/logo.png")).toBe("https://site.com/logo.png");
    });
  });

  describe("toAbsolute", () => {
    it("resolves relative path relative to project file directory", () => {
      const project = "C:/Users/Admin/Documents/project.glucose";
      expect(toAbsolute("textures/image.png", project)).toBe(
        "C:/Users/Admin/Documents/textures/image.png",
      );
      expect(toAbsolute("../image.png", project)).toBe(
        "C:/Users/Admin/image.png",
      );
    });

    it("resolves file:// relative path", () => {
      const project = "C:/Users/Admin/Documents/project.glucose";
      expect(toAbsolute("file://textures/image.png", project)).toBe(
        "file://C:/Users/Admin/Documents/textures/image.png",
      );
    });

    it("leaves absolute and web paths unchanged", () => {
      const project = "C:/Users/Admin/Documents/project.glucose";
      expect(toAbsolute("D:/textures/image.png", project)).toBe(
        "D:/textures/image.png",
      );
      expect(toAbsolute("https://site.com/logo.png", project)).toBe(
        "https://site.com/logo.png",
      );
    });
  });
});
