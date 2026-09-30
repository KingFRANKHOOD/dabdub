import { Injectable, UnauthorizedException } from '@nestjs/common';
import { PassportStrategy } from '@nestjs/passport';
import { Strategy } from 'passport-custom';
import { Request } from 'express';
import { AuthService } from '../auth.service';

@Injectable()
export class ApiKeyStrategy extends PassportStrategy(Strategy, 'api-key') {
  constructor(private readonly authService: AuthService) {
    super();
  }

  async validate(req: Request) {
    const rawKey = req.headers['x-api-key'];
    if (typeof rawKey !== 'string' || !rawKey.trim()) {
      throw new UnauthorizedException('Missing API key');
    }
    const merchant = await this.authService.findMerchantByApiKey(rawKey.trim());
    if (!merchant) throw new UnauthorizedException('Invalid API key');
    return { merchantId: merchant.id, email: merchant.email, role: merchant.role };
  }
}
