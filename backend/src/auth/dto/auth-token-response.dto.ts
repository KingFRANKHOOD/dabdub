import { ApiProperty } from '@nestjs/swagger';
import { Merchant } from '../../merchants/entities/merchant.entity';

export class AuthTokenResponseDto {
  @ApiProperty({ description: 'JWT access token' })
  accessToken: string;

  @ApiProperty({ description: 'Merchant profile (passwordHash and apiKeyHash excluded)' })
  merchant: Omit<Merchant, 'passwordHash' | 'apiKeyHash'>;
}
