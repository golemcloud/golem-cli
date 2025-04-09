import * as z from "zod";

const MethodPattern = z.enum([
  "Get",
  "Post",
  "Put",
  "Delete",
  "Patch",
  "Head",
  "Options",
  "Trace",
  "Connect",
]);

export const BindingType = z.enum(["default", "file-server", "cors-preflight"]);

const GatewayBindingData = z.object({
  bindingType: BindingType,
  component: z
    .object({
      name: z.string(),
      version: z.number(),
    })
    .optional(),
  workerName: z.string().optional(),
  idempotencyKey: z.string().optional(),
  response: z.string().optional(),
});

const HttpCors = z.object({
  allowOrigin: z.string(),
  allowMethods: z.string(),
  allowHeaders: z.string(),
  exposeHeaders: z.string().optional(),
  maxAge: z.number().optional(),
  allowCredentials: z.boolean().optional(),
});

export const RouteRequestData = z.object({
  method: MethodPattern,
  path: z.string(),
  binding: GatewayBindingData,
  cors: HttpCors.optional(),
  security: z.string().optional(),
});