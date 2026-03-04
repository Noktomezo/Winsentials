import { z } from 'zod'

export const TweakUiTypeSchema = z.enum(['toggle', 'radio', 'dropdown', 'multiselect'])
export const TweakCategorySchema = z.enum(['system', 'appearance', 'privacy', 'network', 'input', 'security', 'hardware', 'memory'])
export const RiskLevelSchema = z.enum(['low', 'medium', 'high'])

export const TweakOptionSchema = z.object({
  value: z.string(),
  label_key: z.string(),
  is_default: z.boolean().default(false),
  is_recommended: z.boolean().default(false),
})

export const TweakMetaSchema = z.object({
  id: z.string(),
  category: TweakCategorySchema,
  name_key: z.string(),
  description_key: z.string(),
  details_key: z.string(),
  risk_details_key: z.string().optional(),
  ui_type: TweakUiTypeSchema,
  options: z.array(TweakOptionSchema).default([]),
  requires_reboot: z.boolean().default(false),
  requires_logout: z.boolean().default(false),
  risk_level: RiskLevelSchema.default('low'),
  min_windows_build: z.number().nullable().optional(),
})

export const TweakStateSchema = z.object({
  id: z.string(),
  current_value: z.string().nullable(),
  is_applied: z.boolean(),
})

export const TweakInfoSchema = z.object({
  meta: TweakMetaSchema,
  state: TweakStateSchema,
  is_available: z.boolean(),
  windows_version_required: z.string().nullable(),
})

export type TweakUiType = z.infer<typeof TweakUiTypeSchema>
export type TweakCategory = z.infer<typeof TweakCategorySchema>
export type RiskLevel = z.infer<typeof RiskLevelSchema>
export type TweakOption = z.infer<typeof TweakOptionSchema>
export type TweakMeta = z.infer<typeof TweakMetaSchema>
export type TweakState = z.infer<typeof TweakStateSchema>
export type TweakInfo = z.infer<typeof TweakInfoSchema>
