import { SnakeCasedProperties } from 'type-fest';
import { EachYellowPagesSettings } from './Settings';

export type YPConfigParams = Omit<EachYellowPagesSettings, 'host'>;
export type YPConfigParamTypes = SnakeCasedProperties<YPConfigParams>;
export type YPConfigParam = keyof YPConfigParamTypes;

export default interface YPConfig {
  name: string;
  termsURL: string;
  termsSelector: string | null;
  ignoreTermsCheck: boolean;
  host: string;
  supportIpv6: boolean;
  prefixHeader: string;
  supportedParams: readonly YPConfigParam[];
}
