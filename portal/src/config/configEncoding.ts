import { Base64 } from "js-base64";

import {
  safeBase64Decode,
  safeJSONParse,
  SafeParseResult,
} from "../utils/safeParse";
import {
  PortalRunConfig,
  PortalRunConfigStringy,
  portalRunConfigStringySchema,
} from "./config";

export const encodeConfigInURL = (
  config: PortalRunConfig | PortalRunConfigStringy,
  baseHref = window.location.href
): string => {
  const url = new URL(baseHref);
  const stringyConfig = portalRunConfigStringySchema.parse(config);
  url.searchParams.set(
    "runConfig",
    Base64.encode(JSON.stringify(stringyConfig))
  );
  return url.toString();
};

export const decodeConfigFromURL = (
  href = window.location.href
): SafeParseResult<PortalRunConfigStringy> => {
  const url = new URL(href);
  const encodedConfig = url.searchParams.get("runConfig");
  if (!encodedConfig) return { success: false };
  const decoded = safeBase64Decode(encodedConfig);
  if (!decoded.success) return decoded;
  const parsed = safeJSONParse(decoded.data);
  if (!parsed.success) return parsed;
  const checked = portalRunConfigStringySchema.safeParse(parsed.data);
  if (!checked.success) return checked;
  return { success: true, data: checked.data };
};
