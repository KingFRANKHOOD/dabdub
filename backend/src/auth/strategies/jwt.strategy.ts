import { Injectable, UnauthorizedException } from '@nestjs/common';
import { PassportStrategy } from '@nestjs/passport';
import { ExtractJwt, Strategy } from 'passport-jwt';
import { ConfigService } from '@nestjs/config';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Merchant } from '../../merchants/entities/merchant.entity';

@Injectable()
export class JwtStrategy extends PassportStrategy(Strategy) {
  constructor(
    config: ConfigService,
    @InjectRepository(Merchant)
    private readonly merchantsRepo: Repository<Merchant>,
  ) {
    super({
      jwtFromRequest: ExtractJwt.fromAuthHeaderAsBearerToken(),
      ignoreExpiration: false,
      secretOrKey: config.get<string>('JWT_SECRET', 'fallback-secret'),
    });
  }

  async validate(payload: { sub: string; email: string; role: string }) {
    const merchant = await this.merchantsRepo.findOne({ where: { id: payload.sub } });
    if (!merchant) throw new UnauthorizedException('Merchant not found');
    return { merchantId: merchant.id, email: merchant.email, role: merchant.role };
  }
}
