import { Injectable } from '@nestjs/common';
import { AuthGuard } from '@nestjs/passport';

/** Apply to public-facing API routes that accept x-api-key authentication. */
@Injectable()
export class ApiKeyGuard extends AuthGuard('api-key') {}
