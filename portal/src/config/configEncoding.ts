import { Base64 } from "js-base64";

import {
  safeBase64Decode,
  safeJSONParse,
  SafeParseResult,
} from "../utils/safeParse";
import {
  PortalRunConfigStringy,
  portalRunConfigStringySchema,
} from "./PortalRunConfig";

export const encodeConfigInURL = (
  config: PortalRunConfigStringy,
  baseHref = window.location.href
): string => {
  const url = new URL(baseHref);
  url.searchParams.set("runConfig", Base64.encode(JSON.stringify(config)));
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
